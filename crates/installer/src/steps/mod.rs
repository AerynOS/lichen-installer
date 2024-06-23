// SPDX-FileCopyrightText: Copyright © 2024 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Lichen installation steps

mod context;
pub use context::Context;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io: {0}")]
    IO(#[from] std::io::Error),
}

#[derive(Debug)]
pub enum Step<'a> {
    Format(Box<partitions::FormatPartition<'a>>),
    Mount(Box<partitions::MountPartition<'a>>),
}

impl<'a> Step<'a> {
    /// Return a unique short ID name for the steps
    pub fn name(&self) -> &'static str {
        match &self {
            Step::Format(_) => "format-partition",
            Step::Mount(_) => "mount-partition",
        }
    }

    /// Return the display title for a step
    pub fn title(&self) -> String {
        match &self {
            Step::Format(s) => s.title(),
            Step::Mount(s) => s.title(),
        }
    }

    /// Describe the action/context for the step
    pub fn describe(&self) -> String {
        match &self {
            Step::Format(s) => s.describe(),
            Step::Mount(s) => s.describe(),
        }
    }

    /// Execute a step asynchronously. Implementations can opt-in to async.
    pub async fn execute(&self, context: &mut Context) -> Result<(), Error> {
        match &self {
            Step::Format(s) => Ok(s.execute(context).await?),
            Step::Mount(s) => Ok(s.execute(context).await?),
        }
    }
}

mod partitions;
use std::fmt::Debug;

pub use partitions::{FormatPartition, MountPartition};
