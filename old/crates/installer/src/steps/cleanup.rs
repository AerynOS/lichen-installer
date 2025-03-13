// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Lichen cleanup steps
//! Despite the fact we could trivially implement as a `Drop` or some
//! other Rust idiom, we still need to provide active feedback to the user
//! for whatever step is currently running.
//!
//! To that effect we provide a mirror of [`Step`] by way of a Cleanup.

use super::{partitions, Context};

/// Encapsulate the cleanup stages
pub enum Cleanup {
    /// Unmount a mountpoint
    Unmount(Box<partitions::Unmount>),

    /// Sync filesystems pre unmount
    Sync(Box<partitions::SyncFS>),
}

impl<'a> Cleanup {
    /// Create new unmount cleanup stage
    pub fn unmount(unmount: partitions::Unmount) -> Self {
        Self::Unmount(Box::new(unmount))
    }

    /// Create new sync helper
    pub fn sync_fs() -> Self {
        Self::Sync(Box::new(partitions::SyncFS {}))
    }

    /// Return cleanup step title
    pub fn title(&self) -> String {
        match &self {
            Self::Unmount(s) => s.title(),
            Self::Sync(s) => s.title(),
        }
    }

    /// Fully describe cleanup step
    pub fn describe(&self) -> String {
        match &self {
            Self::Unmount(s) => s.describe(),
            Self::Sync(s) => s.describe(),
        }
    }

    /// Execute the cleanup step
    pub fn execute(&self, context: &impl Context<'a>) -> Result<(), super::Error> {
        match &self {
            Self::Unmount(s) => Ok(s.execute(context)?),
            Self::Sync(s) => Ok(s.execute(context)?),
        }
    }
}
