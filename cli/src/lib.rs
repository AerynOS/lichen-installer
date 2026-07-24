// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use color_eyre::eyre;
use installer::{DisplayInfo, Installer, Model, Step};
use protocols::lichen::osinfo::OsInfo;

pub mod args;
pub mod frontend;
pub mod install_model;
pub mod logging;
pub mod selections;

pub enum FrontendStep {
    Storage,
    Locale,
    Timezone,
    Desktop,
    Accounts,
    Summary,
}

impl FrontendStep {
    async fn run(&self, info: &OsInfo, installer: &Installer, model: &mut Model) -> eyre::Result<()> {
        match self {
            Self::Storage => frontend::storage::run(info, installer, model).await?,
            Self::Locale => frontend::locale::run(installer, model).await?,
            Self::Timezone => frontend::timezone::run(installer, model).await?,
            Self::Desktop => frontend::desktop::run(installer, model).await?,
            Self::Accounts => frontend::accounts::run(installer, model).await?,
            Self::Summary => frontend::summary::run(installer, model).await?,
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
