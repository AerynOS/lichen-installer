// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Reading and writing the AerynOS `system-model.kdl`
//!
//! The emitted document carries the moss-owned `repositories` and `packages`
//! nodes plus a lichen-owned `installer` node holding the choices moss cannot
//! express, partitoning strategy, locale, timezone, accounts. moss ignores
//! unknown top-level nodes and preserves them verbatim across transactions,
//! so both live in the same file.
//!
//! Passwords appear only as crypt(3) hashes.

use std::fs;

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
    let mut doc = KdlDocument::new();

    doc.nodes_mut().push(repositories_node());
    doc.nodes_mut().push(packages_node(&model.software.packages));
    doc.nodes_mut().push(installer_node(model));
    doc.autoformat();
    doc.to_string()
}

/// Parse a previously emitted model, toleratnly: absent nodes leave the
/// corresponding fields at their defaults so the steps prompt as usual.
pub fn from_kdl(content: &str) -> Result<Model, KdlError> {
    let doc: KdlDocument = content.parse()?;
    let mut model = Model::default();

    if let Some(packages) = doc.get("packages") {
        model.software.packages = packages
            .iter_children()
            .map(|node| node.name().value().to_string())
            .collect();
    }

    if let Some(installer) = doc.get("installer") {
        for child in installer.iter_children() {
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
    }
    Ok(model)
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

    let mut volatile = KdlNode::new("volatile");
    let mut volatile_children = KdlDocument::new();
    let mut description = KdlNode::new("description");

    description.push(KdlEntry::new("AerynOS volatile package stream"));
    volatile_children.nodes_mut().push(description);

    let mut base_uri = KdlNode::new("base-uri");
    base_uri.push(KdlEntry::new("https://build.aerynos.dev/"));
    volatile_children.nodes_mut().push(base_uri);

    let mut version = KdlNode::new("version");
    version.push(KdlEntry::new("stream/volatile"));
    volatile_children.nodes_mut().push(version);

    let mut priority = KdlNode::new("priority");
    priority.push(KdlEntry::new(0i128));
    volatile_children.nodes_mut().push(priority);

    volatile.set_children(volatile_children);

    let mut node = KdlNode::new("repositories");
    let mut children = KdlDocument::new();
    children.nodes_mut().push(volatile);
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

/// The lichen-owned installer node: everything moss cannot express
fn installer_node(model: &Model) -> KdlNode {
    let mut node = KdlNode::new("installer");
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

    node.set_children(children);
    node
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
    fn moss_owned_nodes_are_present_and_valid() {
        let text = to_kdl(&sample_model());
        let doc: KdlDocument = text.parse().expect("emitted model must be valid KDL");

        assert!(doc.get("repositories").is_some());
        assert!(doc.get("packages").is_some());
        assert!(doc.get("installer").is_some());
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
                volatile {
                    uri "https://build.aerynos.dev/stream/volatile/x86_64/stone.index"
                    priority 0
                }
                rooted {
                    base-uri "https://build.aerynos.dev/"
                    version "stream/volatile"
                }
            }
        "#;

        let repos = repositories(text).expect("must parse");

        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].id, "volatile");
        assert_eq!(
            repos[0].uri,
            "https://build.aerynos.dev/stream/volatile/x86_64/stone.index"
        );
    }
}
