// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use disks::BlockDevice;
use protocols::proto_disks::{self, disks_server, ListDisksRequest, ListDisksResponse};
use tonic::{Request, Response};

/// Service represents the disk management service implementation
#[derive(Debug, Default)]
pub struct Service {}

/// Creates a new Disks gRPC server instance using the default Service implementation
pub fn service() -> disks_server::DisksServer<Service> {
    disks_server::DisksServer::new(Service::default())
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
    async fn list_disks(
        &self,
        _request: Request<ListDisksRequest>,
    ) -> Result<Response<ListDisksResponse>, tonic::Status> {
        // Discover all block devices on the system
        let devices = BlockDevice::discover()?;

        // Filter and transform block devices into disk information
        let disks = devices
            .iter()
            .filter_map(|device| match device {
                BlockDevice::Disk(disk) => Some(proto_disks::Disk {
                    name: device.name().to_owned(),
                    sectors: device.sectors(),
                    device: device.device().to_string_lossy().to_string(),
                    model: disk.model().map(|m| m.to_owned()),
                    vendor: disk.vendor().map(|v| v.to_owned()),
                    partitions: device
                        .partitions()
                        .iter()
                        .map(|partition| proto_disks::Partition {
                            name: partition.name.clone(),
                            number: partition.number,
                            start: partition.start,
                            end: partition.end,
                            size: partition.size,
                            node: partition.node.to_string_lossy().to_string(),
                            device: partition.device.to_string_lossy().to_string(),
                        })
                        .collect(),
                }),
                _ => None,
            })
            .collect();

        let response = ListDisksResponse { disks };
        Ok(Response::new(response))
    }
}
