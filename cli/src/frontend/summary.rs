// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use super::storage;
use crate::{CliStep, FrontendStep, system_model};
use installer::{DisplayInfo, Icon, Installer, Model, StepError, register_step};
use protocols::lichen::{install::WriteSystemModelRequest, storage::provisioner::ApplyStrategyRequest};

pub async fn run(installer: &Installer, model: &mut Model) -> Result<(), StepError> {
    let plan = model
        .storage
        .plan
        .as_ref()
        .ok_or_else(|| StepError::Failed("no storage configuration was selected".to_string()))?;
    let mut text = String::new();

    text.push_str(&format!("Target disk:  {}\n", model.storage.disk_display));
    text.push_str(&format!("Strategy:     {}\n", model.storage.strategy_name));
    text.push_str(&format!("Locale:       {}\n", model.region.language));
    text.push_str(&format!("Timezone:     {}\n", model.region.timezone));
    text.push_str(&format!(
        "Desktop:      {} ({}packages)\n",
        model.software.selection,
        model.software.packages.len()
    ));

    match &model.accounts.user {
        Some(user) => text.push_str(&format!("User:      {} ({})\n", user.username, user.real_name)),
        None => text.push_str("User:      not configured\n"),
    }

    text.push_str(&format!(
        "Root account: {}\n",
        if model.accounts.root_password_hash.is_some() {
            "password set"
        } else {
            "not configured"
        }
    ));
    text.push('\n');
    text.push_str(&storage::render_plan(plan));

    cliclack::note("Installation summary", text).map_err(|_| StepError::UserAborted)?;

    let confirmed = cliclack::confirm(format!(
        "Erase {} and install? ALL DATA ON THIS DISK WILL BE DESTROYED.",
        model.storage.disk,
    ))
    .initial_value(false)
    .interact()
    .map_err(|_| StepError::UserAborted)?;

    if !confirmed {
        return Err(StepError::UserAborted);
    }

    let mut provisioner = installer.provisioner().await?;
    let applied = provisioner
        .apply_strategy(ApplyStrategyRequest {
            strategy: model.storage.strategy_id.clone(),
            disks: vec![model.storage.disk.clone()],
        })
        .await?
        .into_inner();
    let applied_plan = applied
        .plan
        .ok_or_else(|| StepError::Failed("backend returned no applied plan".to_string()))?;
    let root_device = applied_plan
        .role_mounts
        .iter()
        .find(|role_mount| role_mount.mountpoint == "/")
        .map(|role_mount| role_mount.device.clone())
        .ok_or_else(|| StepError::Failed("applied plan has no root mount".to_string()))?;
    let contents = system_model::to_kdl(model);
    let mut install = installer.install().await?;
    install
        .write_system_model(WriteSystemModelRequest { root_device, contents })
        .await?;
    let mounts = applied_plan
        .role_mounts
        .iter()
        .map(|role_mount| format!("  {} on {}", role_mount.device, role_mount.mountpoint))
        .collect::<Vec<_>>()
        .join("\n");
    let _ = cliclack::log::success(format!(
        "Disk partitioned and formatted:\n{mounts}\n\nSystem model written to /usr/lib/system-model.kdl on the target root"
    ));

    Ok(())
}

register_step! {
    id: "summary",
    author: "AerynOS Developers",
    description: "Review the installation summary",
    create: || Box::new(CliStep { info: DisplayInfo {
        title: "Summary".to_string(),
        description: "Review the installation summary".to_string(),
        icon: Some(Icon::Emoji("📝".to_string())),
    }, step: FrontendStep::Summary })
}
