// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

mod step;
use std::{
    collections::{BTreeMap, HashSet},
    env,
    path::Path,
    sync::Arc,
};

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
    available_steps: HashSet<String>,
}

/// Builder for Installer
pub struct InstallerBuilder {
    backend_path: Option<Box<Path>>,
    step_ids: Vec<String>,
    active_step: Option<String>,
}

#[derive(Debug, Error)]
pub enum NavigationError {
    #[error("No next step available")]
    NoNextStep,
    #[error("No previous step available")]
    NoPreviousStep,
    #[error("Step {0} not found")]
    StepNotFound(String),
    #[error("Step {0} not available")]
    StepUnavailable(String),
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Failed to connect to the privileged service")]
    ConnectionError(#[from] protocols::Error),

    #[error("Missing backend path")]
    MissingBackendPath,

    #[error("Failed to load step plugin: {0}")]
    StepLoadError(String),

    #[error("Navigation error: {0}")]
    NavigationError(#[from] NavigationError),
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
        for step_id in &self.step_ids {
            let step = get_step(step_id).ok_or_else(|| Error::StepLoadError(step_id.clone()))?;
            steps.insert(step_id.clone(), step);
        }

        let mut available_steps = HashSet::new();
        // By default, make first step available
        if let Some(first_step) = self.step_ids.first() {
            available_steps.insert(first_step.clone());
        }
        Ok(Installer {
            backend_path: str_path,
            steps,
            connection,
            active_step: self.active_step,
            available_steps,
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

    /// Get all step IDs in order
    pub fn step_ids(&self) -> Vec<&String> {
        self.steps.keys().collect()
    }

    /// Make a step available for navigation
    pub fn make_step_available(&mut self, step_id: &str) -> Result<(), NavigationError> {
        if !self.steps.contains_key(step_id) {
            return Err(NavigationError::StepNotFound(step_id.to_string()));
        }
        self.available_steps.insert(step_id.to_string());
        Ok(())
    }

    /// Make a step unavailable
    pub fn make_step_unavailable(&mut self, step_id: &str) {
        self.available_steps.remove(step_id);
    }

    /// Check if a step is available
    pub fn is_step_available(&self, step_id: &str) -> bool {
        self.available_steps.contains(step_id)
    }

    /// Get all available steps
    pub fn available_steps(&self) -> Vec<&String> {
        self.available_steps.iter().collect()
    }

    /// Navigate to a specific step
    pub fn goto_step(&mut self, step_id: &str) -> Result<(), NavigationError> {
        if !self.steps.contains_key(step_id) {
            return Err(NavigationError::StepNotFound(step_id.to_string()));
        }
        if !self.is_step_available(step_id) {
            return Err(NavigationError::StepUnavailable(step_id.to_string()));
        }
        self.set_active_step(step_id);
        Ok(())
    }

    /// Navigate to the next available step
    pub fn next_step(&mut self) -> Result<(), NavigationError> {
        self.active_step = Some(
            self.next_available_step_id()
                .ok_or(NavigationError::NoNextStep)?
                .to_owned(),
        );
        Ok(())
    }

    /// Navigate to the previous available step
    pub fn previous_step(&mut self) -> Result<(), NavigationError> {
        self.active_step = Some(
            self.previous_available_step_id()
                .ok_or(NavigationError::NoPreviousStep)?
                .to_owned(),
        );
        Ok(())
    }

    /// Check if there is a next step available
    pub fn has_next(&self) -> bool {
        self.next_available_step_id().is_some()
    }

    /// Check if there is a previous step available
    pub fn has_previous(&self) -> bool {
        self.previous_available_step_id().is_some()
    }

    // Helper methods to find next/previous available steps
    fn next_available_step_id(&self) -> Option<&String> {
        let current = self.active_step_id()?;
        let ids = self.step_ids();
        let current_idx = ids.iter().position(|id| *id == current)?;

        ids.iter()
            .skip(current_idx + 1)
            .find(|id| self.is_step_available(id))
            .map(|v| &**v)
    }

    fn previous_available_step_id(&self) -> Option<&String> {
        let current = self.active_step_id()?;
        let ids = self.step_ids();
        let current_idx = ids.iter().position(|id| *id == current)?;

        ids.iter()
            .take(current_idx)
            .rev()
            .find(|id| self.is_step_available(id))
            .map(|v| &**v)
    }

    /// Grab a disks RPC client
    pub async fn disks(&self) -> Result<disks_client::DisksClient<Channel>, Error> {
        let channel =
            protocols::service_connection_to_channel(self.connection.clone(), self.backend_path.clone()).await?;
        let client = disks_client::DisksClient::new(channel);
        Ok(client)
    }
}
