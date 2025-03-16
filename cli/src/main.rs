// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use cli::{frontend::Interface, logging::CliclackLayer};
use color_eyre::Result;
use console::style;
use protocols::{privileged::ServiceConnection, proto_disks};
use std::fs::File;
use std::{env, sync::Arc};
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format::Format, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

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
    let file = File::create("installer.log")?;
    let file_format = Format::default()
        .with_ansi(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_file(false)
        .with_line_number(false)
        .with_target(true)
        .with_thread_ids(true);

    let file_filter = EnvFilter::new("trace");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(file_format)
                .with_writer(file)
                .with_filter(file_filter),
        )
        .with(ErrorLayer::default())
        .with(CliclackLayer)
        .init();

    Ok(())
}

async fn test_client(connection: Arc<ServiceConnection>, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let channel = protocols::service_connection_to_channel(connection, path.to_string()).await?;
    let mut client = proto_disks::disks_client::DisksClient::new(channel);
    let disks = client.list_disks(proto_disks::ListDisksRequest {}).await?.into_inner();
    for disk in disks.disks {
        tracing::info!("Disk on backend: {:?}", disk);
    }
    Ok(())
}

fn run_installer() -> Result<()> {
    let installer = Interface::new()?;
    installer.run()?;
    Ok(())
}

// Main entry point
fn main() -> Result<()> {
    setup_eyre();
    configure_tracing()?;

    let our_bin = env::current_exe()?;
    let our_exe = our_bin.with_file_name("lichen_backend");
    let path = our_exe.to_string_lossy().to_string();
    let connection = protocols::create_service_connection(&our_exe)?;

    cliclack::intro(style("  Install AerynOS  ").white().on_magenta().bold())?;

    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to build tokio runtime");

    rt.block_on(async { test_client(connection.clone(), &path).await.unwrap() });

    run_installer()
}
