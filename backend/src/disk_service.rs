// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use disks::BlockDevice;
use lichen_macros::authorized;
use protocols::lichen::disks::{disks_server, ListDisksRequest, ListDisksResponse};
use tonic::{Request, Response};

use crate::auth::AuthService;

/// Service represents the disk management service implementation
#[derive(Debug)]
pub struct Service {
    auth: Arc<AuthService>,
}

/// Creates a new Disks gRPC server instance using the default Service implementation
pub fn service(auth: Arc<AuthService>) -> disks_server::DisksServer<Service> {
    disks_server::DisksServer::new(Service { auth })
}

#[tonic::async_trait]
impl disks_server::Disks for Service {
    /// Lists all available disk devices and their partitions
    ///
    /// # Parameters
    /// * `_request` - The incoming gRPC request (unused)
    ///
    /// # Returns
    /// A Response containing ListDisksResponse with disk information, or a tonic::Status error

    #[authorized("com.aerynos.lichen.disks.list")]
    async fn list_disks(
        &self,
        request: Request<ListDisksRequest>,
    ) -> Result<Response<ListDisksResponse>, tonic::Status> {
        // Discover all block devices on the system
        let devices = BlockDevice::discover()?;

        // Filter and transform block devices into disk information
        let disks = devices
            .iter()
            .filter(|device| {
                if request.get_ref().exclude_loopback {
                    !matches!(device, BlockDevice::Loopback(_))
                } else {
                    true
                }
            })
            .map(Into::into)
            .collect();

        let response = ListDisksResponse { disks };
        Ok(Response::new(response))
    }
}
