// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module

use color_eyre::eyre;
use console::style;
use installer::{Installer, Step};

use crate::CliStep;

pub mod disks;
pub mod summary;

pub struct Frontend {
    pub installer: Installer,
}

impl Frontend {
    // Create a new Frontend instance
    pub fn new(installer: Installer) -> eyre::Result<Self> {
        Ok(Self { installer })
    }

    // Render the current step
    fn render_step(step: &dyn Step) -> eyre::Result<()> {
        cliclack::clear_screen()?;
        let info = step.info();
        let title = style(format!("  {}  ", info.title)).bold();
        let subtitle = style(info.description.clone()).dim();
        cliclack::intro(title)?;
        cliclack::log::remark(subtitle)?;
        Ok(())
    }

    async fn perform_step(&self, step: &dyn Step) -> eyre::Result<()> {
        let cli_step = step
            .as_any()
            .downcast_ref::<CliStep>()
            .ok_or_else(|| eyre::eyre!("Failed to downcast step to CliStep"))?;
        Self::render_step(cli_step)?;

        cli_step.step.run(&self.installer).await?;
        cliclack::outro("")?;
        Ok(())
    }

    // Step through the menu, returning the selected step ID
    async fn step_menu(&mut self) -> eyre::Result<String> {
        cliclack::intro(style("  Install AerynOS  ").white().on_magenta().bold())?;
        cliclack::log::remark("Welcome to the AerynOS installer")?;
        cliclack::log::warning(format!(
            "This is an {} quality installer, use at your own risk!",
            style("alpha").red()
        ))?;

        let step_ids = self.installer.available_steps();
        let steps = step_ids
            .iter()
            .filter_map(|id| {
                let s = self.installer.step(id)?;
                let info = s.info();
                let title = style(info.title.clone()).bold();
                Some((id, title, info.description.clone()))
            })
            .collect::<Vec<_>>();

        let step_id = cliclack::select("Select a step to continue")
            .items(&steps)
            .interact()
            .map_err(|_| eyre::eyre!("User aborted"))?;

        Ok(step_id.to_string())
    }

    async fn run_internal(&mut self) -> eyre::Result<()> {
        loop {
            // Build a virtual menu of steps
            let step_id = self.step_menu().await?;

            self.installer.goto_step(&step_id)?;

            let step = self
                .installer
                .active_step()
                .ok_or_else(|| eyre::eyre!("No active step found in the installer"))?;
            Self::perform_step(self, step).await?;

            if !self.installer.has_next() {
                break;
            }
        }

        // Make the summary step available and go to it
        self.installer.make_step_available("summary")?;
        self.installer.goto_step("summary")?;
        let step = self
            .installer
            .active_step()
            .ok_or_else(|| eyre::eyre!("No active step found in the installer"))?;
        Self::perform_step(self, step).await?;
        Ok(())
    }

    // Run the CLI installer
    pub async fn run(mut self) -> eyre::Result<()> {
        let _ = self.run_internal().await;
        //self.installer.shutdown().await?;
        Ok(())
    }
}
