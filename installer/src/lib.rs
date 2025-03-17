// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

mod step;
use std::{collections::BTreeMap, env, path::Path, sync::Arc};

use protocols::privileged::ServiceConnection;
use protocols::proto_disks::disks_client;
pub use step::*;
mod icon;
pub use icon::*;
mod model;
pub use model::*;

pub use inventory;
use thiserror::Error;
use tonic::transport::Channel;

/// The installer workflow / mechanism
pub struct Installer {
    steps: BTreeMap<String, Box<dyn Step>>,
    connection: Arc<ServiceConnection>,
    backend_path: String,
    active_step: Option<String>,
}

/// Builder for Installer
pub struct InstallerBuilder {
    backend_path: Option<Box<Path>>,
    step_ids: Vec<String>,
    active_step: Option<String>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to connect to the privileged service")]
    ConnectionError(#[from] protocols::Error),

    #[error("Missing backend path")]
    MissingBackendPath,

    #[error("Failed to load step plugin: {0}")]
    StepLoadError(String),
}

impl InstallerBuilder {
    /// Create a new installer builder
    fn new() -> Self {
        Self {
            backend_path: env::current_exe()
                .map(|p| p.with_file_name("lichen_backend").into())
                .ok(),
            step_ids: Vec::new(),
            active_step: None,
        }
    }

    /// Set the backend path
    pub fn backend_path(mut self, path: &Path) -> Self {
        self.backend_path = Some(Box::from(path));
        self
    }

    /// Add a step by ID
    pub fn add_step(mut self, step_id: &str) -> Self {
        self.step_ids.push(step_id.to_string());
        self
    }

    /// Set the active step
    pub fn active_step(mut self, step_id: &str) -> Self {
        self.active_step = Some(step_id.to_string());
        self
    }

    /// Build the installer
    pub async fn build(self) -> Result<Installer, Error> {
        let backend_path = self.backend_path.ok_or_else(|| Error::MissingBackendPath)?;
        let str_path = backend_path.to_string_lossy().to_string();
        let connection = protocols::create_service_connection(&backend_path)?;

        // Here we would load the step plugins based on their IDs
        let mut steps = BTreeMap::new();
        for step_id in self.step_ids {
            let step = get_step(&step_id).ok_or_else(|| Error::StepLoadError(step_id.clone()))?;
            steps.insert(step_id, step);
        }

        Ok(Installer {
            backend_path: str_path,
            steps,
            connection,
            active_step: self.active_step,
        })
    }
}

impl Installer {
    /// Create a new installer builder
    pub fn builder() -> InstallerBuilder {
        InstallerBuilder::new()
    }

    /// Get the active step ID
    pub fn active_step_id(&self) -> Option<&str> {
        self.active_step.as_deref()
    }

    /// Get the active step
    pub fn active_step(&self) -> Option<&dyn Step> {
        self.active_step_id()
            .and_then(|id| self.steps.get(id).map(|s| s.as_ref()))
    }

    /// Set the active step
    pub fn set_active_step(&mut self, step_id: &str) {
        self.active_step = Some(step_id.to_string());
    }

    /// Grab a disks RPC client
    pub async fn disks(&self) -> Result<disks_client::DisksClient<Channel>, Error> {
        let channel =
            protocols::service_connection_to_channel(self.connection.clone(), self.backend_path.clone()).await?;
        let client = disks_client::DisksClient::new(channel);
        Ok(client)
    }
}
