// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module for disk selection during installation
//!
//! This module provides the disk selection step of the installation process,
//! allowing users to choose which disk to install AerynOS on.

use async_trait::async_trait;
use installer::{register_step, DisplayInfo, Installer, Step, StepError};
use protocols::proto_disks::{Disk, ListDisksRequest};

/// Represents the disk selection installation step
pub struct DiskStep {
    /// Display information for this step
    info: DisplayInfo,
}

#[async_trait]
impl Step for DiskStep {
    /// Returns the display information for this step
    fn info(&self) -> &DisplayInfo {
        &self.info
    }

    /// Implements Any for this step
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    /// Implements mutable Any for this step
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    /// Executes the disk selection step
    async fn run(&self, installer: &Installer) -> Result<(), StepError> {
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
            .map(|(idx, d)| (idx, Self::render_disk(d), "".to_string()))
            .collect::<Vec<_>>();

        let _index = cliclack::select("What disk would you like to install AerynOS on?")
            .items(&renderable_devices)
            .interact()
            .map_err(|_| StepError::UserAborted)?;

        Ok(())
    }
}

impl DiskStep {
    fn render_disk(disk: &Disk) -> String {
        format!(
            "{} - {} - {}",
            disk.name,
            disk.model.as_deref().unwrap_or("Unknown"),
            disk.display_size
        )
    }

    fn new() -> Self {
        Self {
            info: DisplayInfo {
                title: "Disk selection".to_string(),
                description: "Select the disk to install AerynOS on".to_string(),
                icon: None,
            },
        }
    }
}

register_step! {
    id: "disks",
    author: "AerynOS Developers",
    description: "Select the disk to install AerynOS on",
    create: || Box::new(DiskStep::new())
}
