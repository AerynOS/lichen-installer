// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module

use color_eyre::eyre;
use console::style;
use installer::{Installer, Step};
use protocols::lichen::osinfo::OsInfo;

use crate::CliStep;

pub mod locale;
pub mod storage;
pub mod summary;

pub struct Frontend {
    pub installer: Installer,
    pub info: OsInfo,
}

impl Frontend {
    // Create a new Frontend instance
    pub fn new(installer: Installer, info: OsInfo) -> eyre::Result<Self> {
        Ok(Self { installer, info })
    }

    // Render the current step
    fn render_step(step: &dyn Step) -> eyre::Result<()> {
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

        cli_step.step.run(&self.info, &self.installer).await?;
        cliclack::outro("")?;
        Ok(())
    }

    fn render_intro(&self) -> eyre::Result<()> {
        let identity = self.info.metadata.as_ref().and_then(|m| m.identity.as_ref());
        let os_name = identity.map(|i| i.display.clone()).unwrap_or("Unknown OS".into());
        let proj_name = identity.map(|i| i.name.clone()).unwrap_or("Unknown NAME".into());

        let color_string = identity.and_then(|i| i.ansi_color.clone()).unwrap_or("1;36".into());
        let styled = format!("\x1b[{color_string}m  Install {os_name}   ");
        cliclack::intro(styled)?;

        cliclack::log::remark(format!("Welcome to the {} installer", proj_name))?;
        cliclack::log::warning(format!(
            "This is an {} quality installer, use at your own risk!",
            style("alpha").red()
        ))?;

        Ok(())
    }

    async fn run_internal(&mut self) -> eyre::Result<()> {
        // Render the intro
        self.render_intro()?;

        // Process all intermediate steps
        loop {
            let step = self
                .installer
                .active_step()
                .ok_or_else(|| eyre::eyre!("No active step found in the installer"))?;
            Self::perform_step(self, step).await?;

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
        Self::perform_step(self, step).await?;
        Ok(())
    }

    // Run the CLI installer
    pub async fn run(mut self) -> eyre::Result<()> {
        self.run_internal().await?;
        Ok(())
    }
}
