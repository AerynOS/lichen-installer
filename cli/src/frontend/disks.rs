// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module for disk selection during installation
//!
//! This module provides the disk selection step of the installation process,
//! allowing users to choose which disk to install AerynOS on.

use installer::{register_step, DisplayInfo, Installer, StepError};
use protocols::proto_disks::{Disk, ListDisksRequest};

use crate::{CliStep, FrontendStep};

pub async fn run(installer: &Installer) -> Result<(), StepError> {
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

    let _index = cliclack::select("What disk would you like to install AerynOS on?")
        .items(&renderable_devices)
        .interact()
        .map_err(|_| StepError::UserAborted)?;

    Ok(())
}

fn render_disk(disk: &Disk) -> String {
    format!(
        "{} - {} - {}",
        disk.name,
        disk.model.as_deref().unwrap_or("Unknown"),
        disk.display_size
    )
}

register_step! {
    id: "disks",
    author: "AerynOS Developers",
    description: "Select the disk to install AerynOS on",
    create: || Box::new(CliStep { info: DisplayInfo {
        title: "Disk selection".to_string(),
        description: "Select the disk to install AerynOS on".to_string(),
        icon: None,
    }, step: FrontendStep::Disks })
}
