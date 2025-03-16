// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Interface module

use color_eyre::Result;
use protocols::proto_disks::{disks_client::DisksClient, Disk, ListDisksRequest};
use tonic::transport::Channel;

pub struct Interface {
    pub disks: DisksClient<Channel>,
}

impl Interface {
    // Create a new Interface instance
    pub fn new(disks: DisksClient<Channel>) -> Result<Self> {
        Ok(Self { disks })
    }

    fn render_disk(disk: &Disk) -> String {
        format!("{}: {}", disk.device, disk.name)
    }

    pub async fn run(&mut self) -> Result<()> {
        let disks = self.disks.list_disks(ListDisksRequest {}).await?.into_inner();
        let renderable_devices = disks
            .disks
            .iter()
            .enumerate()
            .map(|(idx, d)| (idx, Self::render_disk(d), "".to_string()))
            .collect::<Vec<_>>();

        let _index = cliclack::select("What disk would you like to install AerynOS on?")
            .items(&renderable_devices)
            .interact()?;

        Ok(())
    }
}
