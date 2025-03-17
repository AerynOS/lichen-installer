// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

/// An icon to display for a step
pub enum Icon {
    /// A standard icon (free desktop icon name)
    Standard(String),

    /// A symbolic icon (free desktop icon name)
    Symbolic {
        /// The icon name
        name: String,
        /// The fallback icon name
        fallback: String,
    },

    /// A path to a custom icon, with a specified size
    Custom(String, u32, u32),
}
