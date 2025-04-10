// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use lichen_macros::authorized;
use protocols::lichen::system::{system_server, SystemShutdownResponse, SystemStatusResponse};
use tokio::sync::mpsc::UnboundedSender;
use tonic::Request;
use tonic::Response;
use tracing::warn;

use crate::auth::AuthService;

/// System service for queries and shutdown
#[derive(Debug)]
pub struct Service {
    start_time: std::time::Instant,
    sender: UnboundedSender<()>,
    auth: Arc<AuthService>,
}

/// Creates a new gRPC server instance using the default Service implementation
pub fn service(auth: Arc<AuthService>, sender: UnboundedSender<()>) -> system_server::SystemServer<Service> {
    system_server::SystemServer::new(Service {
        start_time: std::time::Instant::now(),
        sender,
        auth,
    })
}

#[tonic::async_trait]
impl system_server::System for Service {
    #[authorized("com.aerynos.lichen.system.status")]
    async fn status(&self, request: Request<()>) -> Result<Response<SystemStatusResponse>, tonic::Status> {
        let uptime = self.start_time.elapsed().as_secs();
        let response = SystemStatusResponse { uptime };
        Ok(Response::new(response))
    }

    /// Shutdown the system service
    #[authorized("com.aerynos.lichen.system.shutdown")]
    async fn shutdown(&self, request: Request<()>) -> Result<Response<SystemShutdownResponse>, tonic::Status> {
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
        let osinf = os_info::load_os_info_from_path("/usr/lib/os-info.json")
            .map_err(|e| tonic::Status::internal(format!("Failed to load OS info: {}", e)))?;

        Ok(Response::new(osinf.into()))
    }
}
