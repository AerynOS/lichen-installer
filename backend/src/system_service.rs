// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use protocols::lichen::system::{system_server, SystemShutdownResponse, SystemStatusResponse};
use tokio::sync::mpsc::UnboundedSender;
use tonic::Request;
use tonic::Response;
use tracing::warn;

/// System service for queries and shutdown
#[derive(Debug)]
pub struct Service {
    start_time: std::time::Instant,
    sender: UnboundedSender<()>,
}

/// Creates a new gRPC server instance using the default Service implementation
pub fn service(sender: UnboundedSender<()>) -> system_server::SystemServer<Service> {
    system_server::SystemServer::new(Service {
        start_time: std::time::Instant::now(),
        sender,
    })
}

#[tonic::async_trait]
impl system_server::System for Service {
    async fn status(&self, _request: Request<()>) -> Result<Response<SystemStatusResponse>, tonic::Status> {
        let uptime = self.start_time.elapsed().as_secs();
        let response = SystemStatusResponse { uptime };
        Ok(Response::new(response))
    }

    /// Shutdown the system service
    async fn shutdown(&self, _request: Request<()>) -> Result<Response<SystemShutdownResponse>, tonic::Status> {
        let response = SystemShutdownResponse { shutting_down: true };

        // Send a signal to the parent process to shut down
        self.sender.send(()).unwrap();

        warn!("Shutting down the backend service");

        Ok(Response::new(response))
    }

    /// Get the OS information
    async fn get_os_info(
        &self,
        _request: Request<()>,
    ) -> Result<Response<protocols::lichen::osinfo::OsInfo>, tonic::Status> {
        Err(tonic::Status::unimplemented("Not yet implemented"))
    }
}
