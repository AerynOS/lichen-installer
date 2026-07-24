// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::{CliStep, FrontendStep, selections};
use installer::{DisplayInfo, Icon, Installer, Model, StepError, register_step};

pub async fn run(_installer: &Installer, model: &mut Model) -> Result<(), StepError> {
    if model.imported && !model.region.timezone.is_empty() {
        let _ = cliclack::log::info(format!(
            "Using imported desktop environment {} with {} packages",
            model.software.selection,
            model.software.packages.len(),
        ));
        return Ok(());
    }

    let d_envs = selections::desktops();
    let items = d_envs
        .iter()
        .map(|sel| (sel.name.clone(), sel.summary.clone(), sel.description.clone()))
        .collect::<Vec<_>>();
    let picked: String = cliclack::select("Select your desktop environment")
        .items(&items)
        .interact()
        .map_err(|_| StepError::UserAborted)?;
    let packages = selections::resolve(&picked)?;

    tracing::info!("Selected DE {picked} with {} packages", packages.len());
    model.software.selection = picked;
    model.software.packages = packages;

    Ok(())
}

register_step! {
    id: "desktop",
    author: "AerynOS Developers",
    description: "Select the desktop experience",
    create: || Box::new(
        CliStep {
            info: DisplayInfo {
                title: "Desktop".to_string(),
                description: "Select the desktop environment".to_string(),
                icon: Some(Icon::Emoji("💻".to_string())),
            },
            step: FrontendStep::Desktop,
        }
    )
}
