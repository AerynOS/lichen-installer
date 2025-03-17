// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use protocols::proto_backend::{
    backend_server, BackendShutdownRequest, BackendShutdownResponse, BackendStatusRequest, BackendStatusResponse,
};
use tokio::sync::mpsc::UnboundedSender;
use tonic::Request;
use tonic::Response;
use tracing::warn;

/// Service represents the disk management service implementation
#[derive(Debug)]
pub struct Service {
    start_time: std::time::Instant,
    sender: UnboundedSender<()>,
}

/// Creates a new Disks gRPC server instance using the default Service implementation
pub fn service(sender: UnboundedSender<()>) -> backend_server::BackendServer<Service> {
    backend_server::BackendServer::new(Service {
        start_time: std::time::Instant::now(),
        sender,
    })
}

#[tonic::async_trait]
impl backend_server::Backend for Service {
    async fn status(
        &self,
        _request: Request<BackendStatusRequest>,
    ) -> Result<Response<BackendStatusResponse>, tonic::Status> {
        let uptime = self.start_time.elapsed().as_secs();
        let response = BackendStatusResponse { uptime };
        Ok(Response::new(response))
    }

    /// Shutdown the backend service
    async fn shutdown(
        &self,
        _request: Request<BackendShutdownRequest>,
    ) -> Result<Response<BackendShutdownResponse>, tonic::Status> {
        let response = BackendShutdownResponse { shutting_down: true };

        // Send a signal to the parent process to shut down
        self.sender.send(()).unwrap();

        warn!("Shutting down the backend service");

        Ok(Response::new(response))
    }
}
