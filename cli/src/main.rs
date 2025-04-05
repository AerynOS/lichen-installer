// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use cli::{frontend::Frontend, logging::CliclackLayer};
use color_eyre::Result;
use installer::Installer;
use protocols::lichen::locales::GetLocaleRequest;
use std::env;
use std::fs::File;
use tracing::info;
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

// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    setup_eyre();
    configure_tracing()?;

    let mut installer = Installer::builder()
        .add_step("disks")
        .add_step("summary")
        .active_step("disks")
        .build()
        .await?;

    // Make the first step available
    installer.make_step_available("disks")?;

    let mut system = installer.system().await?;
    let info = system.get_os_info(()).await?;

    let mut locale = installer.locales().await?;
    for locale in locale.list_locales(()).await?.into_inner().locales {
        info!("Available locale: {} {}", locale.display_name, locale.name);
    }

    let system_locale = locale
        .get_locale(GetLocaleRequest {
            name: env::var("LANG").unwrap_or("en_US.utf-8".to_string()),
        })
        .await?
        .into_inner();

    info!(
        "System locale currently set to {} {:?}",
        system_locale.display_name,
        system_locale.territory.map(|t| t.flag)
    );

    let iface = Frontend::new(installer, info.into_inner())?;
    iface.run().await?;
    Ok(())
}
