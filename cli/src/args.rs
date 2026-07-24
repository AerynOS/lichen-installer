// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Command line arguments and turning them into an imported model

use crate::{
    install_model::{apply_install_model, apply_system_model, is_install_model, parse_error_detail},
    selections::mandatory,
};
use clap::{CommandFactory, Parser, error::ErrorKind};
use color_eyre::Result;
use installer::Model;
use kdl::KdlError;
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Import installer settings from an install-model.kdl
    #[arg(short, long, value_name = "PATH")]
    install_model: Option<PathBuf>,
    /// Import the packages from a system-model.kdl
    #[arg(short, long, value_name = "PATH")]
    system_model: Option<PathBuf>,
}

impl Args {
    /// Load an imported model from whichever documents were given.
    pub fn model(&self) -> Result<Option<Model>, clap::Error> {
        if self.install_model.is_none() && self.system_model.is_none() {
            return Ok(None);
        }

        let mut model = Model::default();

        if let Some(path) = &self.install_model {
            let contents = read_document(path)?;

            if !is_install_model(&contents).map_err(|e| parse_failure(path, &e))? {
                return Err(invalid_value(format!(
                    "{} has no install-model block; pass a bare system model with --system-model",
                    path.display()
                )));
            }

            apply_install_model(&mut model, &contents).map_err(|e| parse_failure(path, &e))?;
        }

        if let Some(path) = &self.system_model {
            let contents = read_document(path)?;

            if is_install_model(&contents).map_err(|e| parse_failure(path, &e))? {
                return Err(invalid_value(format!(
                    "{} is an install-model, not a system model; pass it with --install-model",
                    path.display()
                )));
            }

            apply_system_model(&mut model, &contents).map_err(|e| parse_failure(path, &e))?;
        }

        model.imported = true;
        ensure_mandatory(&mut model)?;
        Ok(Some(model))
    }
}

/// An imported model never install lass than a bootable system
fn ensure_mandatory(model: &mut Model) -> Result<(), clap::Error> {
    let mut packages: BTreeSet<String> = model.software.packages.iter().cloned().collect();
    packages.extend(mandatory(&model.software.selection).map_err(|e| invalid_value(e.to_string()))?);
    model.software.packages = packages.into_iter().collect();
    Ok(())
}

/// Read a model document, naming the file when it cannot be read
fn read_document(path: &Path) -> Result<String, clap::Error> {
    fs::read_to_string(path)
        .map_err(|e| Args::command().error(ErrorKind::Io, format!("failed to read {}: {e}", path.display())))
}

/// A KDL parse failure, with the location within the document
fn parse_failure(path: &Path, error: &KdlError) -> clap::Error {
    invalid_value(format!(
        "failed to parse {}: {}",
        path.display(),
        parse_error_detail(error)
    ))
}

/// A legit flag error, not to emit a debug error
fn invalid_value(message: String) -> clap::Error {
    Args::command().error(ErrorKind::InvalidValue, message)
}
