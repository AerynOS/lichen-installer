// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module

use color_eyre::Result;
use console::style;
use installer::Installer;
use protocols::proto_disks::{Disk, ListDisksRequest};

pub struct Frontend {
    pub installer: Installer,
}

impl Frontend {
    // Create a new Frontend instance
    pub fn new(installer: Installer) -> Result<Self> {
        Ok(Self { installer })
    }

    // Render a disk with improved styling
    fn render_disk(disk: &Disk) -> String {
        let display_name = match disk.model {
            Some(ref model) => model.clone(),
            None => "Unknown".to_string(),
        };
        let display_vendor = match disk.vendor {
            Some(ref vendor) => vendor.clone(),
            None => "Unknown".to_string(),
        };
        format!(
            "{} {} • {} {}",
            style(&disk.device).cyan().bold(),
            style(&disk.display_size).yellow(),
            style(&display_vendor).dim(),
            style(&display_name).green()
        )
    }

    // Run the CLI installer
    pub async fn run(&self) -> Result<()> {
        cliclack::intro(style("  Install AerynOS  ").white().on_magenta().bold())?;

        // Grab the list of disks
        let mut client = self.installer.disks().await?;
        let disks = client
            .list_disks(ListDisksRequest { exclude_loopback: true })
            .await?
            .into_inner();

        // Ask the user to select a disk
        let _ = self.ask_for_disk(&disks.disks).await?;

        Ok(())
    }

    // Ask the user to select a disk
    async fn ask_for_disk(&self, disks: &[Disk]) -> Result<usize> {
        let renderable_devices = disks
            .iter()
            .enumerate()
            .map(|(idx, d)| (idx, Self::render_disk(d), "".to_string()))
            .collect::<Vec<_>>();

        let _index = cliclack::select("What disk would you like to install AerynOS on?")
            .items(&renderable_devices)
            .interact()?;

        Ok(_index)
    }
}
