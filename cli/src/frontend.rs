// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module

use color_eyre::eyre;
use console::style;
use installer::Installer;

mod disks;
mod summary;

pub struct Frontend {
    pub installer: Installer,
}

impl Frontend {
    // Create a new Frontend instance
    pub fn new(installer: Installer) -> eyre::Result<Self> {
        Ok(Self { installer })
    }

    // Run the CLI installer
    pub async fn run(&mut self) -> eyre::Result<()> {
        cliclack::intro(style("  Install AerynOS  ").white().on_magenta().bold())?;

        loop {
            let step = self
                .installer
                .active_step()
                .ok_or_else(|| eyre::eyre!("No active step found in the installer"))?;

            step.run(&self.installer).await?;
            if !self.installer.has_next() {
                break;
            }
            self.installer.next_step()?;
        }

        // Make the summary step available and go to it
        self.installer.make_step_available("summary")?;
        self.installer.goto_step("summary")?;
        let step = self
            .installer
            .active_step()
            .ok_or_else(|| eyre::eyre!("No active step found in the installer"))?;
        step.run(&self.installer).await?;
        Ok(())
    }
}
