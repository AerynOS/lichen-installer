// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module for disk selection during installation
//!
//! This module provides the disk selection step of the installation process,
//! allowing users to choose which disk to install AerynOS on.

use installer::{register_step, DisplayInfo, Icon, Installer, StepError};
use protocols::lichen::{
    osinfo::OsInfo,
    storage::{
        disks::{Disk, ListDisksRequest},
        provisioner::TryStrategyRequest,
    },
};

use crate::{CliStep, FrontendStep};

pub async fn run(info: &OsInfo, installer: &Installer) -> Result<(), StepError> {
    // Grab the list of disks
    let mut client = installer.disks().await?;
    let disks = client
        .list_disks(ListDisksRequest { exclude_loopback: true })
        .await?
        .into_inner();

    let renderable_devices = disks
        .disks
        .iter()
        .enumerate()
        .map(|(idx, d)| (idx, render_disk(d), "".to_string()))
        .collect::<Vec<_>>();

    let os_name = info
        .metadata
        .as_ref()
        .and_then(|m| m.identity.as_ref())
        .map(|i| i.display.clone())
        .unwrap_or("Unknown OS".into());

    let index = cliclack::select(format!("What disk would you like to install {os_name} on?"))
        .items(&renderable_devices)
        .interact()
        .map_err(|_| StepError::UserAborted)?;

    let selected_disk = disks.disks.get(index).ok_or(StepError::UserAborted)?;
    tracing::info!("Selected disk: {:?}", selected_disk.device);

    let mut provisioner = installer.provisioner().await?;
    let strategies = provisioner.list_strategies(()).await?.into_inner().strategies;
    for s in &strategies {
        tracing::info!("Computing disk strategy: {:?}", s);
        let _plan = provisioner
            .try_strategy(TryStrategyRequest {
                strategy: s.name.clone(),
                disks: vec![selected_disk.device.clone()],
            })
            .await?
            .into_inner();
    }

    Ok(())
}

fn render_disk(disk: &Disk) -> String {
    format!(
        "{} - {} - {}",
        disk.device,
        disk.model.as_deref().unwrap_or("Unknown"),
        disk.display_size
    )
}

register_step! {
    id: "storage",
    author: "AerynOS Developers",
    description: "Select the disk to install on",
    create: || Box::new(CliStep { info: DisplayInfo {
        title: "Configure storage".to_string(),
        description: "Select the disk to install on".to_string(),
        icon: Some(Icon::Emoji("💾".to_string())),
    }, step: FrontendStep::Storage })
}
