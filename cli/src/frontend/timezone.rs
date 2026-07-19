// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::{CliStep, FrontendStep};
use chrono_tz::TZ_VARIANTS;
use installer::{DisplayInfo, Icon, Installer, Model, StepError, register_step};

pub async fn run(_installer: &Installer, model: &mut Model) -> Result<(), StepError> {
    if model.imported && !model.region.timezone.is_empty() {
        let _ = cliclack::log::info(format!("Using imported timezone {}", model.region.timezone));
        return Ok(());
    }
    let items = TZ_VARIANTS
        .iter()
        .map(|tz| (tz.to_string(), tz.to_string(), ""))
        .collect::<Vec<_>>();
    let picked = cliclack::select("Select your timezone")
        .items(&items)
        .initial_value(model.region.timezone.clone())
        .filter_mode()
        .set_size(12)
        .interact()
        .map_err(|_| StepError::UserAborted)?;

    tracing::info!("Selected timezone {picked}");
    model.region.timezone = picked;

    Ok(())
}

register_step! {
    id: "timezone",
    author: "AerynOS Developers",
    description: "Select the system timezone",
    create: || Box::new(
        CliStep {
            info: DisplayInfo {
                title: "Timezone".to_string(),
                description: "Adjust the system timezone".to_string(),
                icon: Some(Icon::Emoji("🕒".to_string())),
            },
            step: FrontendStep::Timezone,
        }
    )
}
