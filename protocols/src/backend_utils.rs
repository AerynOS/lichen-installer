// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::ops::Deref;

use disks::BlockDevice;

use crate::proto_disks;

/// Converts a disks::partition::Partition reference into a proto_disks::Partition
///
/// Creates a new Partition protobuf message from a disk partition, copying over all relevant fields
/// including name, number, start/end sectors, size, node path and device path.
impl<T> From<T> for proto_disks::Partition
where
    T: Deref<Target = disks::partition::Partition>,
{
    fn from(partition: T) -> Self {
        proto_disks::Partition {
            name: partition.name.clone(),
            number: partition.number,
            start: partition.start,
            end: partition.end,
            size: partition.size,
            node: partition.node.to_string_lossy().to_string(),
            device: partition.device.to_string_lossy().to_string(),
        }
    }
}

/// Converts a BlockDevice reference into a proto_disks::Disk
///
/// Creates a new Disk protobuf message from either a physical disk or loopback device.
/// For physical disks, copies name, sectors, device path, model and vendor info, and all partitions.
/// For loopback devices, copies the same fields but accesses partition info through the backing disk.
impl<T> From<T> for proto_disks::Disk
where
    T: Deref<Target = BlockDevice>,
{
    fn from(device: T) -> Self {
        match &*device {
            BlockDevice::Disk(ref disk) => proto_disks::Disk {
                name: device.name().to_owned(),
                sectors: device.sectors(),
                device: device.device().to_string_lossy().to_string(),
                model: disk.model().map(|m| m.to_owned()),
                vendor: disk.vendor().map(|v| v.to_owned()),
                partitions: device.partitions().iter().map(Into::into).collect(),
            },
            BlockDevice::Loopback(ref loopback) => proto_disks::Disk {
                name: device.name().to_owned(),
                sectors: device.sectors(),
                device: device.device().to_string_lossy().to_string(),
                model: loopback.disk().and_then(|d| d.model()).map(|m| m.to_owned()),
                vendor: loopback.disk().and_then(|d| d.vendor()).map(|v| v.to_owned()),
                partitions: loopback
                    .disk()
                    .map(|d| d.partitions())
                    .unwrap_or_default()
                    .iter()
                    .map(Into::into)
                    .collect(),
            },
        }
    }
}
