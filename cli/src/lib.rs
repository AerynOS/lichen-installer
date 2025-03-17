// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use color_eyre::eyre;
use installer::{DisplayInfo, Installer, Step};

pub mod frontend;
pub mod logging;

pub enum FrontendStep {
    Disks,
    Summary,
}

impl FrontendStep {
    async fn run(&self, installer: &Installer) -> eyre::Result<()> {
        match self {
            Self::Disks => frontend::disks::ask_for_disk(installer).await?,
            Self::Summary => frontend::summary::run_summary(installer).await?,
        }
        Ok(())
    }
}

pub struct CliStep {
    pub info: DisplayInfo,
    pub step: FrontendStep,
}

#[async_trait]
impl Step for CliStep {
    fn info(&self) -> &DisplayInfo {
        &self.info
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
