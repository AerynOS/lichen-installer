// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module

use color_eyre::eyre;
use console::style;
use installer::Installer;

mod disks;

pub struct Frontend {
    pub installer: Installer,
}

impl Frontend {
    // Create a new Frontend instance
    pub fn new(installer: Installer) -> eyre::Result<Self> {
        Ok(Self { installer })
    }

    // Run the CLI installer
    pub async fn run(&self) -> eyre::Result<()> {
        cliclack::intro(style("  Install AerynOS  ").white().on_magenta().bold())?;

        let step = self
            .installer
            .active_step()
            .ok_or_else(|| eyre::eyre!("No active step found in the installer"))?;

        step.run(&self.installer).await?;

        Ok(())
    }
}
