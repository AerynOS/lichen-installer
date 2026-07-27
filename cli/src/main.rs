// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use clap::Parser;
use cli::{args::Args, frontend::Frontend, logging::CliclackLayer};
use color_eyre::Result;
use installer::Installer;
use std::{env, fs::File};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, Layer, fmt::format::Format, layer::SubscriberExt, util::SubscriberInitExt};

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

// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    setup_eyre();
    configure_tracing()?;

    // Reject a bad model path before standing up the backend connection
    let model = args.model().unwrap_or_else(|error| error.exit()).unwrap_or_default();

    let mut installer = Installer::builder()
        .add_step("storage")
        .add_step("locale")
        .add_step("timezone")
        .add_step("desktop")
        .add_step("accounts")
        .add_step("summary")
        .active_step("storage")
        .build()
        .await?;

    // Make every choice step available; summary is unlocked by the
    // frontend once the other steps have run
    installer.make_step_available("storage")?;
    installer.make_step_available("locale")?;
    installer.make_step_available("timezone")?;
    installer.make_step_available("desktop")?;
    installer.make_step_available("accounts")?;

    let mut system = installer.system().await?;
    let info = system.get_os_info(()).await?;

    let iface = Frontend::new(installer, info.into_inner())?;

    iface.run(model).await?;
    Ok(())
}
