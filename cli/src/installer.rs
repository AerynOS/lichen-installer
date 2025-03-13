// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! installer module

use color_eyre::Result;
use disks::BlockDevice;
use tracing::{error, info, trace};

pub struct Installer {
    pub devices: Vec<BlockDevice>,
}

// If the blockdevice is not a loop device, it is usable
// for our purposes
fn is_disk_usable(device: &BlockDevice) -> bool {
    !matches!(device, BlockDevice::Loopback(_))
}

// Discover all block devices and filter out the ones
// that are usable for our purposes
fn usable_disks() -> Result<Vec<BlockDevice>> {
    trace!("Discovering block devices");
    let pbar = cliclack::spinner();
    pbar.start("Discovering block devices");

    match BlockDevice::discover() {
        Ok(devices) => {
            let devices: Vec<BlockDevice> = devices.into_iter().filter(is_disk_usable).collect();
            pbar.clear();
            info!("Found {} usable block devices", devices.len());
            Ok(devices)
        }
        Err(e) => {
            pbar.clear();
            error!("Error discovering block devices: {}", e);
            Ok(vec![])
        }
    }
}

impl Installer {
    // Create a new installer instance
    pub fn new() -> Result<Self> {
        let devices = usable_disks()?;
        Ok(Self { devices })
    }
}
