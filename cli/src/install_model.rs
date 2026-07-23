// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Reading and writing AerynOS installation models
//!
//! Two documents, per the upstream design:
//! - `system-model.kdl` - moss's agnostic system definition
//! - `install-model.kdl` - the installer's strict superset: installer
//!   sections (strategy, disk, locale, timezone, desktop, accounts as crypt
//!   hashes, install date) wrapping a nested `system-model` node. Written to
//!   /etc/moss/install-model.kdl as the permanent installation record;
//!   re-importing it reproduces the installation.
//!
//! The installer is thereby a function from system-model to install-model: a
//! bare system-model can be ingested (packages) and decorated interactively
//! into a full install-model.

use std::fs;

use chrono::Utc;
use installer::{Model, User};
use kdl::{KdlDocument, KdlEntry, KdlError, KdlNode};

pub const LIVE_MODEL_PATH: &str = "/usr/lib/system-model.kdl";

/// A repository definition extracted from a system model document
pub struct Repository {
    pub id: String,
    pub uri: String,
}

/// Extract directly-addressable repos carrying a `uri` node
/// from a system model document
pub fn repositories(content: &str) -> Result<Vec<Repository>, KdlError> {
    let doc: KdlDocument = content.parse()?;
    let mut repos = Vec::new();

    if let Some(node) = doc.get("repositories") {
        for repo in node.iter_children() {
            if let Some(uri) = repo.children().and_then(|child| child.get("uri")).and_then(first_arg) {
                repos.push(Repository {
                    id: repo.name().value().to_string(),
                    uri: uri.to_string(),
                });
            }
        }
    }

    Ok(repos)
}

/// Serialize the collected installation model to KDL text
pub fn to_kdl(model: &Model) -> String {
    let mut children = KdlDocument::new();

    let mut push_arg = |name: &str, value: &str| {
        if !value.is_empty() {
            let mut child = KdlNode::new(name);
            child.push(KdlEntry::new(value));
            children.nodes_mut().push(child);
        }
    };

    push_arg("strategy", &model.storage.strategy_id);
    push_arg("disk", &model.storage.disk);
    push_arg("locale", &model.region.language);
    push_arg("timezone", &model.region.timezone);
    push_arg("desktop", &model.software.selection);
    push_arg("installed", &Utc::now().to_rfc3339());

    let mut accounts = KdlNode::new("accounts");
    let mut account_children = KdlDocument::new();

    if let Some(hash) = &model.accounts.root_password_hash {
        let mut root = KdlNode::new("root");
        root.push(KdlEntry::new_prop("hash", hash.as_str()));
        account_children.nodes_mut().push(root);
    }

    if let Some(user) = &model.accounts.user {
        let mut user_node = KdlNode::new("user");
        user_node.push(KdlEntry::new_prop("name", user.username.as_str()));
        user_node.push(KdlEntry::new_prop("realname", user.real_name.as_str()));
        user_node.push(KdlEntry::new_prop("hash", user.password_hash.as_str()));
        account_children.nodes_mut().push(user_node);
    }

    if !account_children.nodes().is_empty() {
        accounts.set_children(account_children);
        children.nodes_mut().push(accounts);
    }

    let mut system = KdlNode::new("system-model");
    system.set_children(system_model_document(model));
    children.nodes_mut().push(system);

    let mut root = KdlNode::new("install-model");
    root.set_children(children);

    let mut doc = KdlDocument::new();
    doc.nodes_mut().push(root);
    doc.autoformat();
    doc.to_string()
}

/// Parse a previously emitted model, toleratnly: absent nodes leave the
/// corresponding fields at their defaults so the steps prompt as usual.
pub fn from_kdl(content: &str) -> Result<Model, KdlError> {
    let doc: KdlDocument = content.parse()?;
    let mut model = Model::default();

    if let Some(install) = doc.get("install-model") {
        for child in install.iter_children() {
            apply_installer_field(&mut model, child);
        }

        if let Some(system) = install.children().and_then(|children| children.get("system-model"))
            && let Some(inner) = system.children()
        {
            apply_packages(&mut model, inner);
        } else {
            apply_packages(&mut model, &doc);
        }
    }

    Ok(model)
}

/// Apply one installer-section field to the model
fn apply_installer_field(model: &mut Model, child: &KdlNode) {
    match child.name().value() {
        "strategy" => {
            if let Some(value) = first_arg(child) {
                model.storage.strategy_id = value.to_string();
                model.storage.strategy_name = value.to_string();
            }
        }
        "disk" => {
            if let Some(value) = first_arg(child) {
                model.storage.disk = value.to_string();
            }
        }
        "locale" => {
            if let Some(value) = first_arg(child) {
                model.region.language = value.to_string();
            }
        }
        "timezone" => {
            if let Some(value) = first_arg(child) {
                model.region.timezone = value.to_string();
            }
        }
        "desktop" => {
            if let Some(value) = first_arg(child) {
                model.software.selection = value.to_string();
            }
        }
        "accounts" => {
            for account in child.iter_children() {
                match account.name().value() {
                    "root" => {
                        model.accounts.root_password_hash = prop(account, "hash").map(str::to_string);
                    }
                    "user" => {
                        if let (Some(name), Some(hash)) = (prop(account, "name"), prop(account, "hash")) {
                            model.accounts.user = Some(User {
                                username: name.to_string(),
                                real_name: prop(account, "realname").unwrap_or_default().to_string(),
                                password_hash: hash.to_string(),
                            });
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// Ingest the packages node from a system-model document
fn apply_packages(model: &mut Model, doc: &KdlDocument) {
    if let Some(packages) = doc.get("packages") {
        model.software.packages = packages
            .iter_children()
            .map(|node| node.name().value().to_string())
            .collect();
    }
}

/// The moss-owned repo node: cloned from the live system model when
/// running on AerynOS media, otherwise a built-in default template
fn repositories_node() -> KdlNode {
    if let Ok(content) = fs::read_to_string(LIVE_MODEL_PATH)
        && let Ok(doc) = content.parse::<KdlDocument>()
        && let Some(node) = doc.get("repositories")
    {
        return node.clone();
    }

    let mut unstable = KdlNode::new("unstable");
    let mut unstable_children = KdlDocument::new();
    let mut description = KdlNode::new("description");

    description.push(KdlEntry::new("AerynOS unstable package stream"));
    unstable_children.nodes_mut().push(description);

    let mut base_uri = KdlNode::new("base-uri");
    base_uri.push(KdlEntry::new("https://cdn.aerynos.dev/"));
    unstable_children.nodes_mut().push(base_uri);

    let mut version = KdlNode::new("version");
    version.push(KdlEntry::new("stream/unstable"));
    unstable_children.nodes_mut().push(version);

    let mut priority = KdlNode::new("priority");
    priority.push(KdlEntry::new(0i128));
    unstable_children.nodes_mut().push(priority);

    unstable.set_children(unstable_children);

    let mut node = KdlNode::new("repositories");
    let mut children = KdlDocument::new();
    children.nodes_mut().push(unstable);
    node.set_children(children);

    node
}

/// The moss-owned packages node, one child node per package/provider
fn packages_node(packages: &[String]) -> KdlNode {
    let mut node = KdlNode::new("packages");
    let mut children = KdlDocument::new();

    packages.iter().for_each(|pkg| {
        children.nodes_mut().push(KdlNode::new(pkg.as_str()));
    });

    node.set_children(children);
    node
}

/// Serialize the bare, moss-pure system-model
pub fn system_model_kdl(model: &Model) -> String {
    let mut doc = system_model_document(model);
    doc.autoformat();
    doc.to_string()
}

/// The moss-owned system-model.kdl
fn system_model_document(model: &Model) -> KdlDocument {
    let mut doc = KdlDocument::new();
    doc.nodes_mut().push(repositories_node());
    doc.nodes_mut().push(packages_node(&model.software.packages));
    doc
}

/// First positional argument of a node, as a str
fn first_arg(node: &KdlNode) -> Option<&str> {
    node.entries()
        .iter()
        .find(|entry| entry.name().is_none())
        .and_then(|entry| entry.value().as_string())
}

/// Named property of a node, as a str
pub(crate) fn prop<'a>(node: &'a KdlNode, key: &str) -> Option<&'a str> {
    node.get(key).and_then(|value| value.as_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_model() -> Model {
        let mut model = Model::default();
        model.storage.disk = "/dev/vda".to_string();
        model.storage.strategy_id = "whole_disk".to_string();
        model.storage.strategy_name = "whole_disk".to_string();
        model.region.language = "en_US.UTF-8".to_string();
        model.region.timezone = "America/Los_Angeles".to_string();
        model.software.selection = "gnome".to_string();
        model.software.packages = vec![
            "binary(cc)".to_string(),
            "pkgconfig(zlib)".to_string(),
            "firefox".to_string(),
        ];
        model.accounts.root_password_hash = Some("$6$salt$roothash".to_string());
        model.accounts.user = Some(User {
            username: "john".to_string(),
            real_name: "John Doe".to_string(),
            password_hash: "$6$salt$userhash".to_string(),
        });

        model
    }

    #[test]
    fn round_trip_preserves_choices() {
        let text = to_kdl(&sample_model());
        let parsed = from_kdl(&text).expect("emitted model must parse");

        assert_eq!(parsed.storage.disk, "/dev/vda");
        assert_eq!(parsed.storage.strategy_id, "whole_disk");
        assert_eq!(parsed.region.language, "en_US.UTF-8");
        assert_eq!(parsed.region.timezone, "America/Los_Angeles");
        assert_eq!(parsed.software.selection, "gnome");
        assert_eq!(
            parsed.software.packages,
            vec![
                "binary(cc)".to_string(),
                "pkgconfig(zlib)".to_string(),
                "firefox".to_string(),
            ]
        );
        assert_eq!(parsed.accounts.root_password_hash.as_deref(), Some("$6$salt$roothash"));

        let user = parsed.accounts.user.expect("user must round trip");
        assert_eq!(user.username, "john");
        assert_eq!(user.real_name, "John Doe");
        assert_eq!(user.password_hash, "$6$salt$userhash");
    }

    #[test]
    fn document_forms_are_correct() {
        let full = to_kdl(&sample_model());
        let full_doc: KdlDocument = full.parse().expect("install-model must be valid KDL");

        assert!(full_doc.get("install-model").is_some());
        assert!(full_doc.get("packages").is_none());

        let bare = system_model_kdl(&sample_model());
        let bare_doc: KdlDocument = bare.parse().expect("system-model must be valid KDL");

        assert!(bare_doc.get("repositories").is_some());
        assert!(bare_doc.get("packages").is_some());
        assert!(bare_doc.get("install-model").is_none());
    }

    #[test]
    fn empty_model_still_produces_valid_kdl() {
        let text = to_kdl(&Model::default());
        let parsed = from_kdl(&text).expect("emtpy model must round trip");
        assert!(parsed.software.packages.is_empty());
        assert!(parsed.accounts.user.is_none());
    }

    #[test]
    fn repositories_extracts_direct_urls() {
        let text = r#"
            repositories {
                unstable {
                uri "https://build.aerynos.dev/stream/unstable/x86_64/stone.index"
                    priority 0
                }
                rooted {
                    base-uri "https://build.aerynos.dev/"
                    version "stream/unstable"
                }
            }
        "#;

        let repos = repositories(text).expect("must parse");

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].id, "unstable");
        assert_eq!(
            repos[0].uri,
            "https://build.aerynos.dev/stream/unstable/x86_64/stone.index"
        );
    }
}
