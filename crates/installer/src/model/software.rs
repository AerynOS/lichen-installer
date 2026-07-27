// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

/// Software installation settings
#[derive(Debug, Default)]
pub struct Model {
    /// Name of the chosen desktop environment
    pub selection: String,
    /// Fully resolved package/provider list for the target installation
    pub packages: Vec<String>,
}
