// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! The disk service backend
//!
//! This service handles disk management operations and provides a gRPC interface
//! for clients to interact with disk devices.

use std::{env, fs::File};

use backend::{backend_service, disk_service};
use protocols::privileged::{service_init, ServiceListener};
use tokio::net::UnixListener;
use tokio::signal::unix::{signal, SignalKind};
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

use color_eyre::Result;
pub use protocols::proto_disks;
use tracing::info;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt::format::Format, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

/// Configures color-eyre for enhanced error handling and reporting
///
/// Sets up error hooks with metadata about the environment and package version
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

/// Configures the tracing system for application logging
///
/// Creates a log file at "backend.log" and sets up formatting and filtering options
/// for the tracing subscriber
fn configure_tracing() -> Result<()> {
    let file = File::create("backend.log")?;
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
        .init();

    Ok(())
}

/// Handles termination signals (SIGTERM and SIGINT)
///
/// Waits for either signal and returns when one is received, triggering
/// graceful shutdown
async fn signal_handler() {
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigterm.recv() => {},
        _ = sigint.recv() => {},
    };
}

/// Main entry point for the disk service
///
/// Initializes the service, sets up error handling and logging, and starts
/// the gRPC server with the disk service implementation. Handles graceful
/// shutdown on termination signals.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    service_init()?;

    setup_eyre();
    configure_tracing()?;

    let listener = ServiceListener::new()?;
    listener.set_nonblocking(true)?;
    let as_tokio = UnixListener::from_std(listener.0)?;
    let uds_stream = UnixListenerStream::new(as_tokio);

    Server::builder()
        .add_service(disk_service::service())
        .add_service(backend_service::service())
        .serve_with_incoming_shutdown(uds_stream, signal_handler())
        .await?;

    info!("Shutting down");

    Ok(())
}
