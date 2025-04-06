// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::env;

use installer::{register_step, DisplayInfo, Installer, StepError};

use crate::{CliStep, FrontendStep};

pub async fn run(installer: &Installer) -> Result<(), StepError> {
    let mut locales = installer.locales().await?;
    let locale_list = locales.list_locales(()).await?.into_inner();
    let default_option = env::var("LANG").unwrap_or_else(|_| "en_US.UTF-8".to_string());

    let display_list = locale_list
        .locales
        .iter()
        .map(|l| (l.name.clone(), l.display_name.clone(), ""))
        .collect::<Vec<_>>();

    let picked = cliclack::select("Select your locale")
        .items(&display_list)
        .initial_value(default_option)
        .filter_mode()
        .set_size(12)
        .interact()
        .map_err(|_| StepError::UserAborted)?;

    tracing::info!("Selected locale {picked}");

    Ok(())
}

register_step! {
    id: "locale",
    author: "AerynOS Developers",
    description: "Review the installation summary",
    create: || Box::new(CliStep { info: DisplayInfo {
        title: "Locale".to_string(),
        description: "Adjust the system locale".to_string(),
        icon: None,
    }, step: FrontendStep::Locale })
}
