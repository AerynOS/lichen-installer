// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

mod accounts;
mod region;
mod software;
mod storage;

pub use accounts::User;

/// Installation settings
///
/// We take care to use *copy* semantics in order to avoid any spaghetti code
/// which would then make a separate installer backend a nightmare to implement.
#[derive(Default)]
pub struct Model {
    /// Region specific installation settings
    pub region: region::Model,
    /// Storage and partitioning selections
    pub storage: storage::Model,
    /// Account selections
    pub accounts: accounts::Model,
    /// Software selections
    pub software: software::Model,
    /// An imported model from OS refresh option or --model flag
    pub imported: bool,
}
