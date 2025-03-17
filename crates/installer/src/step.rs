// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::Icon;
use std::collections::HashMap;
use std::sync::OnceLock;
use thiserror::Error;
use tracing::{debug, error, trace};

#[derive(Error, Debug)]
pub enum StepError {
    #[error("Step failed: {0}")]
    Failed(String),

    #[error("Protocol error: {0}")]
    ProtocolError(#[from] tonic::Status),

    #[error("Installer error: {0}")]
    InstallerError(#[from] crate::Error),

    #[error("User aborted installation")]
    UserAborted,
}

/// A single step in the installation process. Each step represents a distinct phase
/// of the installation workflow that the user must complete.
pub trait Step: Send + Sync {
    /// Returns display information for rendering this step in the UI
    fn info(&self) -> &DisplayInfo;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Metadata for registering an installation step
#[derive(Debug, Clone)]
pub struct StepInfo {
    /// Unique identifier for the step
    pub id: &'static str,
    /// Author/maintainer of this step
    pub author: &'static str,
    /// Brief description of the step's purpose
    pub description: &'static str,
    /// Factory function to create a new instance of this step
    pub create: fn() -> Box<dyn Step>,
}

inventory::collect!(StepInfo);

/// Retrieves a step implementation by its unique identifier
///
/// # Arguments
///
/// * `id` - The unique identifier of the step to retrieve
///
/// # Returns
///
/// * `Some(Box<dyn Step>)` if a step with the given ID exists
/// * `None` if no matching step was found
pub fn get_step(id: &str) -> Option<Box<dyn Step>> {
    static INIT: OnceLock<HashMap<&'static str, &StepInfo>> = OnceLock::new();
    let steps = INIT.get_or_init(|| {
        trace!("Initializing installation steps");
        let steps: HashMap<_, _> = inventory::iter::<StepInfo>
            .into_iter()
            .map(|info| {
                debug!(id = info.id, author = info.author, "Registered step");
                (info.id, info)
            })
            .collect();
        debug!(count = steps.len(), "Loaded installation steps");
        steps
    });

    match steps.get(id) {
        Some(info) => {
            debug!(id = id, "Creating step instance");
            Some((info.create)())
        }
        None => {
            error!(id = id, "Step \"{id}\" not found");
            None
        }
    }
}

/// Display information for a step that can be shown in the installer UI.
/// Note that strings may be localized and therefore are unsuitable for comparison.
pub struct DisplayInfo {
    /// The title of the step shown to the user
    pub title: String,

    /// A longer description explaining the purpose of this step
    pub description: String,

    /// Optional icon to visually represent this step
    pub icon: Option<Icon>,
}

/// Macro for registering a new installation step
///
/// # Arguments
///
/// * `id` - Unique identifier for the step
/// * `author` - Author/maintainer of the step
/// * `description` - Brief description of the step's purpose
/// * `create` - Factory function to create new instances of the step
#[macro_export]
macro_rules! register_step {
    (id: $id:expr, author: $author:expr, description: $desc:expr, create: $create:expr) => {
        $crate::inventory::submit! {
            $crate::StepInfo {
                id: $id,
                author: $author,
                description: $desc,
                create: $create,
            }
        }
    };
}
