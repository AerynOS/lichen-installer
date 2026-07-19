// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Embedded package selections for the target installation
//!
//! Selections are the JSON definitions in `data/selections/`, compiled into
//! the binary. Each names its required packages/providers and the other
//! selections it depends on.

use installer::StepError;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap, HashSet};

/// A selection definition loaded from data/selections
#[derive(Debug, Clone, Deserialize)]
pub struct Selection {
    pub name: String,
    pub summary: String,
    pub description: String,
    #[serde(default)]
    pub depends: Vec<String>,
    #[serde(default)]
    pub required: Vec<String>,
}

/// Raw embedded selection documents
const RAW: &[&str] = &[
    include_str!("../../data/selections/base.json"),
    include_str!("../../data/selections/cosmic.json"),
    include_str!("../../data/selections/develop.json"),
    include_str!("../../data/selections/gnome.json"),
    include_str!("../../data/selections/kernel-common.json"),
    include_str!("../../data/selections/kernel-desktop.json"),
];

/// Selections that are always part of an installation and never offered
/// as a user-facing choice.
const IMPLICIT: &[&str] = &["kernel-common", "kernel-desktop"];

/// Selections that are structural rather than
/// user-facing.
const HIDDEN: &[&str] = &["base", "develop", "kernel-common", "kernel-desktop"];

/// Parse all embedded selections
pub fn all() -> Vec<Selection> {
    RAW.iter()
        .map(|raw| serde_json::from_str(raw).expect("embedded selection JSON must be valid"))
        .collect()
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

        packages.extend(selection.required.iter().cloned());
        packages.extend(selection.depends.iter().cloned());
    }

    Ok(packages.into_iter().collect())
}
