// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{env, fs::File};

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

#[derive(Debug, Default)]
struct DiskService {}

#[tonic::async_trait]
impl proto_disks::disks_server::Disks for DiskService {
    async fn list_disks(
        &self,
        request: tonic::Request<proto_disks::ListDisksRequest>,
    ) -> Result<tonic::Response<proto_disks::ListDisksResponse>, tonic::Status> {
        println!("Got a request: {:?}", request);
        let response = proto_disks::ListDisksResponse {
            disks: vec![proto_disks::Disk {
                path: "/dev/sda".to_string(),
            }],
        };
        Ok(tonic::Response::new(response))
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

async fn signal_handler() {
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();

    tokio::select! {
        _ = sigterm.recv() => {},
        _ = sigint.recv() => {},
    };
}

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
        // Add service implementations here with .add_service()
        .add_service(proto_disks::disks_server::DisksServer::new(DiskService::default()))
        .serve_with_incoming_shutdown(uds_stream, signal_handler())
        .await?;

    info!("Shutting down");

    Ok(())
}
