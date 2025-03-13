// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! installer module

use cliclack::note;
use color_eyre::{eyre::eyre, Result};
use disks::BlockDevice;
use miette::GraphicalReportHandler;
use provisioning::{Provisioner, StrategyDefinition};
use tracing::{error, info, trace};

pub struct Installer {
    pub devices: Vec<BlockDevice>,
    pub strategies: Vec<StrategyDefinition>,
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
        let strategies = Self::load_strategies("strategies/use_whole_disk.kdl")?;
        Ok(Self { devices, strategies })
    }

    // Load strategies from a file, Render an error if it fails
    fn load_strategies(path: &str) -> Result<Vec<StrategyDefinition>> {
        let parser = provisioning::Parser::new_for_path(path);
        match parser {
            Ok(parser) => Ok(parser.strategies),
            Err(e) => {
                let handler = GraphicalReportHandler::new()
                    .with_links(true)
                    .with_urls(true)
                    .with_show_related_as_nested(true);
                let mut buf = String::new();
                handler.render_report(&mut buf, &e)?;
                error!("Failed to load strategies: {e}\n{buf}");
                Err(eyre!("Failed to load strategies"))
            }
        }
    }

    // Render a block device to a string
    fn render_device(device: &BlockDevice) -> String {
        match device {
            BlockDevice::Loopback(_) => "Loopback device".to_string(),
            BlockDevice::Disk(disk) => disk.to_string(),
        }
    }

    pub fn run(&self) -> Result<()> {
        let renderable_devices = self
            .devices
            .iter()
            .enumerate()
            .map(|(idx, d)| (idx, Self::render_device(d), "".to_string()))
            .collect::<Vec<_>>();

        let index = cliclack::select("What disk would you like to install AerynOS on?")
            .items(&renderable_devices)
            .interact()?;

        let chosen_device = self.devices.get(index).unwrap();
        let mut provisioner = Provisioner::new();
        provisioner.push_device(chosen_device);
        for strategy in &self.strategies {
            provisioner.add_strategy(strategy);
        }

        let plans = provisioner.plan();
        let plan_index = cliclack::select("Select partitioning plan")
            .items(
                &plans
                    .iter()
                    .enumerate()
                    .map(|(i, p)| (i, &p.strategy.name, &p.strategy.summary))
                    .collect::<Vec<_>>(),
            )
            .interact()?;

        let effort = plans.get(plan_index).unwrap();
        for (_, plan) in effort.device_assignments.iter() {
            note(
                format!("Changes to {:?}", plan.device.device()),
                plan.planner.describe_changes(),
            )?;
        }
        let mut mountpoints = effort
            .role_mounts
            .iter()
            .map(|(role, what)| format!("{:?} on {}", what, role.as_path()))
            .collect::<Vec<_>>();
        mountpoints.sort();
        let string_mountpoints = mountpoints.join("\n");
        note("Mountpoints", &string_mountpoints)?;
        Ok(())
    }
}
