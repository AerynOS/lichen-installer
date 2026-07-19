// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Install service: privileged operations for installing the target system

use crate::auth::AuthService;
use disks::BlockDevice;
use lichen_macros::authorized;
use protocols::lichen::install::{
    DiscoverSystemModelsResponse, DiscoveredModel, WriteSystemModelRequest, WriteSystemModelResponse,
    install_server::{Install, InstallServer},
};
use std::{fs, path::Path, process::Command, sync::Arc};
use tonic::{Request, Response, Status};
use tracing::info;

/// Where the target root is temp mounted while writing
const TARGET_MOUNT: &str = "/run/lichen/target";
// Where candidate partitions are briefly mounted read-only while probing
/// for a previous installation's system model
const PROBE_MOUNT: &str = "/run/lichen/probe";
/// Location of the system model inside the target root; moss reads and
/// rewrites this exact path on the installed system
const MODEL_PATH: &str = "usr/lib/system-model.kdl";

/// Service represents the install service implementation
#[derive(Debug)]
pub struct Service {
    auth: Arc<AuthService>,
}

/// Creates a new Install gRPC server instance using the default Service implementation
pub fn service(auth: Arc<AuthService>) -> InstallServer<Service> {
    InstallServer::new(Service { auth })
}

#[tonic::async_trait]
impl Install for Service {
    #[authorized("com.aerynos.lichen.install.write-model")]
    async fn write_system_model(
        &self,
        request: Request<WriteSystemModelRequest>,
    ) -> Result<Response<WriteSystemModelResponse>, tonic::Status> {
        let req = request.into_inner();
        info!(root_device = %req.root_device, "Writing system model target");

        if req.root_device.is_empty() {
            return Err(Status::invalid_argument("no root device provided"));
        }
        if !Path::new(&req.root_device).exists() {
            return Err(Status::not_found(format!("no such device: {}", req.root_device)));
        }

        tokio::task::block_in_place(|| write_to_target(&req.root_device, &req.contents))?;

        Ok(Response::new(WriteSystemModelResponse {}))
    }

    #[authorized("com.aerynos.lichen.install.discover")]
    async fn discover_system_models(
        &self,
        request: Request<()>,
    ) -> Result<Response<DiscoverSystemModelsResponse>, tonic::Status> {
        let _ = request;
        info!("Probing disks for previous installation system models");
        let models = tokio::task::block_in_place(discover_models)?;

        Ok(Response::new(DiscoverSystemModelsResponse { models }))
    }
}

/// Mount the target root, write the model, and always unmount again,
/// even when the write fails
fn write_to_target(root_device: &str, contents: &str) -> Result<(), Status> {
    let target = Path::new(TARGET_MOUNT);

    fs::create_dir_all(target)?;

    run(Command::new("mount").args([root_device, target.to_str().expect("expected a target")]))?;

    let result = (|| -> Result<(), Status> {
        let model_path = target.join(MODEL_PATH);
        if let Some(parent) = model_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&model_path, contents)?;
        Ok(())
    })();

    let unmounted = run(Command::new("umount").arg(target));

    result.and(unmounted)
}

/// Probe every unmounted partition read-only for a system model from a previous
/// installation. Unmountable partitions are auto skipped.
fn discover_models() -> Result<Vec<DiscoveredModel>, Status> {
    let probe = Path::new(PROBE_MOUNT);
    fs::create_dir_all(probe)?;

    let mounted = fs::read_to_string("/proc/self/mounts").unwrap_or_default();
    let devices = BlockDevice::discover()?;
    let mut models = Vec::new();

    for device in &devices {
        for part in device.partitions() {
            let node = part.device.display().to_string();

            // Never touch partitions that are already mounted somewhere
            if mounted.lines().any(|line| line.starts_with(&format!("{node} "))) {
                continue;
            }

            // No mountable filesystem means not a candidate
            if run(Command::new("mount").args(["-o", "ro", &node, &probe.to_string_lossy()])).is_err() {
                continue;
            }

            let contents = fs::read_to_string(probe.join(MODEL_PATH)).ok();
            let _ = run(Command::new("umount").arg(probe));

            if let Some(contents) = contents {
                info!(device = %node, "Found system model from a previous installation");
                models.push(DiscoveredModel { device: node, contents });
            }
        }
    }

    Ok(models)
}

/// Run a command to completion, mapping failure to a gRPC status carrying
/// the command's stderr
fn run(command: &mut Command) -> Result<(), Status> {
    let output = command
        .output()
        .map_err(|e| Status::internal(format!("failed to spawn {:?}: {e}", command.get_program())))?;
    if !output.status.success() {
        return Err(Status::internal(format!(
            "{:?} failed: {}",
            command.get_program(),
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    Ok(())
}
