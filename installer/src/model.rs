// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

mod region;

/// Installation settings
///
/// We take care to use *copy* semantics in order to avoid any spaghetti code
/// which would then make a separate installer backend a nightmare to implement.
pub struct Model {
    /// Region specific installation settings
    pub region: region::Model,
}
