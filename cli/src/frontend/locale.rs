// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use installer::{DisplayInfo, Icon, Installer, Model, StepError, register_step};

use crate::{CliStep, FrontendStep};

pub async fn run(installer: &Installer, model: &mut Model) -> Result<(), StepError> {
    if model.imported && !model.region.language.is_empty() {
        let _ = cliclack::log::info(format!("Using imported locale {}", model.region.language));
        return Ok(());
    }

    let mut locales = installer.locales().await?;
    let locale_list = locales.list_locales(()).await?.into_inner();
    let display_list = locale_list
        .locales
        .iter()
        .map(|l| (l.name.clone(), l.display_name.clone(), ""))
        .collect::<Vec<_>>();

    let picked = cliclack::select("Select your locale")
        .items(&display_list)
        .filter_mode()
        .set_size(12)
        .interact()
        .map_err(|_| StepError::UserAborted)?;

    tracing::info!("Selected locale {picked}");
    model.region.language = picked;

    Ok(())
}

register_step! {
    id: "locale",
    author: "AerynOS Developers",
    description: "Select the system locale",
    create: || Box::new(CliStep { info: DisplayInfo {
        title: "Locale".to_string(),
        description: "Adjust the system locale".to_string(),
        icon: Some(Icon::Emoji("🌎".to_string())),
    }, step: FrontendStep::Locale })
}
