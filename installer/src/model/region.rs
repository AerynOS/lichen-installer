// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

/// Region specific installation settings
pub struct Model {
    /// System language (i.e. "en_US")
    pub language: String,

    /// System timezone (i.e. "Europe/London")
    pub timezone: String,
}

impl Default for Model {
    fn default() -> Self {
        Self {
            language: String::from("en_US.UTF-8"),
            timezone: String::from("UTC"),
        }
    }
}
