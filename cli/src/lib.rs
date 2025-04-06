// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use color_eyre::eyre;
use installer::{DisplayInfo, Installer, Step};
use protocols::lichen::osinfo::OsInfo;

pub mod frontend;
pub mod logging;

pub enum FrontendStep {
    Storage,
    Locale,
    Summary,
}

impl FrontendStep {
    async fn run(&self, info: &OsInfo, installer: &Installer) -> eyre::Result<()> {
        match self {
            Self::Storage => frontend::storage::run(info, installer).await?,
            Self::Summary => frontend::summary::run(installer).await?,
            Self::Locale => frontend::locale::run(installer).await?,
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
