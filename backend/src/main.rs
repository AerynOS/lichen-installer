// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use protocols::privileged::{service_init, ServiceListener};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::transport::Server;

pub use protocols::proto_disks;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    service_init()?;
    let listener = ServiceListener::new()?;
    listener.set_nonblocking(true)?;
    let as_tokio = UnixListener::from_std(listener.0)?;
    let uds_stream = UnixListenerStream::new(as_tokio);

    Server::builder()
        // Add service implementations here with .add_service()
        .add_service(proto_disks::disks_server::DisksServer::new(DiskService::default()))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}
