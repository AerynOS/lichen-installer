// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use protocols::proto_backend::{backend_server, BackendStatusRequest, BackendStatusResponse};
use tonic::{Request, Response};

/// Service represents the disk management service implementation
#[derive(Debug)]
pub struct Service {
    start_time: std::time::Instant,
}

impl Default for Service {
    fn default() -> Self {
        Self {
            start_time: std::time::Instant::now(),
        }
    }
}

/// Creates a new Disks gRPC server instance using the default Service implementation
pub fn service() -> backend_server::BackendServer<Service> {
    backend_server::BackendServer::new(Service::default())
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
}
