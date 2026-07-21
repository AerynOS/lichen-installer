// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Embedded package selections for the target installation
//!
//! Selections are the KDL definitions in `data/selections/`, compiled into
//! the binary. Each names its required packages/providers and the other
//! selections it depends on.

use crate::install_model::prop;
use installer::StepError;
use kdl::KdlDocument;
use std::collections::{BTreeSet, HashMap, HashSet};

/// A selection definition loaded from data/selections
#[derive(Debug, Clone)]
pub struct Selection {
    pub name: String,
    pub summary: String,
    pub description: String,
    pub depends: Vec<String>,
    pub packages: Vec<String>,
}

/// Raw embedded selection documents
const RAW: &[&str] = &[
    include_str!("../../data/selections/base.kdl"),
    include_str!("../../data/selections/desktop-common.kdl"),
    include_str!("../../data/selections/cosmic.kdl"),
    include_str!("../../data/selections/develop.kdl"),
    include_str!("../../data/selections/gnome.kdl"),
    include_str!("../../data/selections/plasma.kdl"),
    include_str!("../../data/selections/kernel-common.kdl"),
    include_str!("../../data/selections/kernel-desktop.kdl"),
];

/// Selections that are always part of an installation and never offered
/// as a user-facing choice.
const IMPLICIT: &[&str] = &["kernel-common", "kernel-desktop"];

/// Selections that are structural rather than
/// user-facing.
const HIDDEN: &[&str] = &["base", "desktop-common", "develop", "kernel-common", "kernel-desktop"];

/// Parse all embedded selections
pub fn all() -> Vec<Selection> {
    RAW.iter().map(|raw| parse(raw)).collect()
}

/// The user-facing desktop choices
pub fn desktops() -> Vec<Selection> {
    all()
        .into_iter()
        .filter(|de| !HIDDEN.contains(&de.name.as_str()))
        .collect()
}

/// Resolve a chosen selection into the full, sorted package/provider list:
/// the selection itself, its dependency closure, and the implicit selections
pub fn resolve(name: &str) -> Result<Vec<String>, StepError> {
    let selections = all();
    let by_name: HashMap<&str, &Selection> = selections
        .iter()
        .map(|selection| (selection.name.as_str(), selection))
        .collect();
    let mut pending: Vec<String> = IMPLICIT.iter().map(|selection| selection.to_string()).collect();

    pending.push(name.to_string());

    let mut visited = HashSet::new();
    let mut packages = BTreeSet::new();

    while let Some(current) = pending.pop() {
        if !visited.insert(current.clone()) {
            continue;
        }

        let selection = by_name
            .get(current.as_str())
            .ok_or_else(|| StepError::Failed(format!("unknown selection: {current}")))?;

        packages.extend(selection.packages.iter().cloned());
        pending.extend(selection.depends.iter().cloned());
    }

    Ok(packages.into_iter().collect())
}

/// Parse one embedded selection document
fn parse(raw: &str) -> Selection {
    let doc: KdlDocument = raw.parse().expect("embedded selection KDL must be valid");
    let node = doc
        .get("selection")
        .expect("embedded selection must have a selection node");
    let list = |key: &str| -> Vec<String> {
        node.children()
            .and_then(|children| children.get(key))
            .map(|list| {
                list.iter_children()
                    .map(|child| child.name().value().to_string())
                    .collect()
            })
            .unwrap_or_default()
    };

    Selection {
        name: prop(node, "name").expect("selection name is required").to_string(),
        summary: prop(node, "summary").unwrap_or_default().to_string(),
        description: prop(node, "description").unwrap_or_default().to_string(),
        depends: list("depends"),
        packages: list("packages"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_selections_parse() {
        let selections = all();
        assert_eq!(selections.len(), 8);
        assert!(desktops().iter().any(|sel| sel.name == "cosmic"));
        assert!(desktops().iter().all(|sel| !HIDDEN.contains(&sel.name.as_str())));
    }

    #[test]
    fn resolve_includes_dependency_closure() {
        let packages = resolve("cosmic").expect("cosmic must resolve");

        assert!(packages.contains(&"cosmic-comp".to_string()));
        assert!(packages.contains(&"mesa-dri-drivers".to_string()));
        assert!(packages.contains(&"binary(bash)".to_string()));
        assert!(packages.contains(&"linux-desktop".to_string()));
    }

    #[test]
    fn unknown_selection_fails() {
        assert!(resolve("no-such-selection").is_err())
    }
}
