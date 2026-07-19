// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

/// SOftware installation settings
#[derive(Default)]
pub struct Model {
    /// Name of the chosen desktop environment
    pub selection: String,
    /// Fully resolved package/provider list for the target installation
    pub packages: Vec<String>,
}
