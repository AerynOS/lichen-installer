// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Lichen step context

use std::{
    fmt::Debug,
    path::PathBuf,
    process::{Command, Output},
};

/// Context for the steps that are executing
/// The context provides access to the core installation variables as
/// well as simplified paths for executing commands in a consistent
/// fashion.
pub trait Context<'a>: Sized + Debug + Send {
    /// Return the root directory of the installation
    fn root(&'a self) -> &'a PathBuf;

    /// Run the command asynchronously via the context
    fn run_command(&self, cmd: &mut Command) -> Result<(), super::Error>;

    /// Run command, capture the output
    /// Accepts optional string to write as stdin
    fn run_command_captured(&self, cmd: &mut Command, input: Option<&str>) -> Result<Output, super::Error>;
}
