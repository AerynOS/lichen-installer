// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use color_eyre::Result;
use disks::BlockDevice;
use std::env;
use std::fs::File;
use tracing::{error, info, trace};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format::Format, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

// If the blockdevice is not a loop device, it is usable
// for our purposes
fn is_disk_usable(device: &BlockDevice) -> bool {
    !matches!(device, BlockDevice::Loopback(_))
}

// Discover all block devices and filter out the ones
// that are usable for our purposes
fn usable_disks() -> Result<Vec<BlockDevice>> {
    match BlockDevice::discover() {
        Ok(devices) => {
            let devices: Vec<BlockDevice> = devices.into_iter().filter(is_disk_usable).collect();
            info!("Found {} usable block devices", devices.len());
            Ok(devices)
        }
        Err(e) => {
            error!("Error discovering block devices: {}", e);
            Ok(vec![])
        }
    }
}

// Setup eyre for better error handling
fn setup_eyre() {
    console::set_colors_enabled(true);
    color_eyre::config::HookBuilder::default()
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
        .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
        .add_issue_metadata("os", env::consts::OS)
        .add_issue_metadata("arch", env::consts::ARCH)
        .issue_filter(|_| true)
        .install()
        .unwrap();
}

// Configure tracing for logging
// Now we dump to both output and file
fn configure_tracing() -> Result<()> {
    let console_format = Format::default()
        .with_ansi(true)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_file(false)
        .with_line_number(false)
        .with_target(true)
        .with_thread_ids(false);

    let file = File::create("installer.log")?;
    let file_format = Format::default()
        .with_ansi(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_file(false)
        .with_line_number(false)
        .with_target(true)
        .with_thread_ids(true);

    let console_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let file_filter = EnvFilter::new("trace");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(console_format)
                .with_filter(console_filter),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(file_format)
                .with_writer(file)
                .with_filter(file_filter),
        )
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

// Main entry point
fn main() -> Result<()> {
    setup_eyre();
    configure_tracing()?;

    trace!("Probing disks");
    let disks = usable_disks()?;
    for disk in disks {
        println!("Disk: {disk:?}");
    }
    println!("Hello, world!");

    Ok(())
}
