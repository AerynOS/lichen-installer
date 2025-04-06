// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt::{Display, Formatter};

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

    /// An emoji icon
    Emoji(String),

    /// A path to a custom icon, with a specified size
    Custom(String, u32, u32),
}

impl Display for Icon {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Icon::Standard(name) => write!(f, "{}", name),
            Icon::Symbolic { name, fallback } => write!(f, "{} (fallback: {})", name, fallback),
            Icon::Custom(path, width, height) => write!(f, "{} ({}x{})", path, width, height),
            Icon::Emoji(emoji) => write!(f, "{}", emoji),
        }
    }
}
