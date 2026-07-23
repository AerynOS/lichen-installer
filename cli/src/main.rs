// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use cli::install_model;
use cli::{frontend::Frontend, logging::CliclackLayer};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use installer::{Installer, Model};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, fs};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, Layer, fmt::format::Format, layer::SubscriberExt, util::SubscriberInitExt};

// Setup eyre for better error handling
fn setup_eyre() {
    console::set_colors_enabled(true);
    color_eyre::config::HookBuilder::default()
        .issue_url(concat!(env!("CARGO_PKG_REPOSITORY"), "/issues/new"))
        .add_issue_metadata("version", env!("CARGO_PKG_VERSION"))
        .add_issue_metadata("os", env::consts::OS)
        .add_issue_metadata("arch", env::consts::ARCH)
        .issue_filter(|_| true)
        .install()
        .unwrap();
}

// Configure tracing for logging
// Now we dump to both output and file
fn configure_tracing() -> Result<()> {
    let file = File::create("installer.log")?;
    let file_format = Format::default()
        .with_ansi(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_file(false)
        .with_line_number(false)
        .with_target(true)
        .with_thread_ids(true);

    let file_filter = EnvFilter::new("trace");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(file_format)
                .with_writer(file)
                .with_filter(file_filter),
        )
        .with(ErrorLayer::default())
        .with(CliclackLayer)
        .init();

    Ok(())
}

// Value of the --model flag when given
fn model_arg() -> Result<Option<Model>> {
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--model" {
            return match args.next() {
                Some(path) => Ok(Some(load_model(Path::new(&path))?)),
                None => Err(eyre!("--model requires a path to a model file or directory")),
            };
        }
    }

    Ok(None)
}

/// Load a model from a single document, or from a directory holding
/// install-model.kdl and/or system-model.kdl. With both documents the
/// install-model supplies the installer fields and the system-model
/// supplies the package set.
fn load_model(path: &Path) -> Result<Model> {
    if !path.is_dir() {
        let contents = fs::read_to_string(path)?;
        let mut model = install_model::from_kdl(&contents).map_err(|e| {
            eyre!(
                "failed to parse {}: {}",
                path.display(),
                install_model::parse_error_detail(&e)
            )
        })?;
        model.imported = true;
        return Ok(model);
    }

    let install_record = find_doc(path, "etc/moss/install-model.kdl", "install-model.kdl");
    let system_model = find_doc(path, "usr/lib/system-model.kdl", "system-model.kdl");

    if install_record.is_none() && system_model.is_none() {
        return Err(eyre!(
            "no install-model.kdl or system-model.kdl found under {}",
            path.display()
        ));
    }

    let mut model = Model::default();

    if let Some(file) = install_record {
        let contents = fs::read_to_string(&file)?;

        install_model::apply_install_model(&mut model, &contents).map_err(|e| {
            eyre!(
                "failed to parse {}: {}",
                file.display(),
                install_model::parse_error_detail(&e)
            )
        })?;
    }

    if let Some(file) = system_model {
        let contents = fs::read_to_string(&file)?;

        install_model::apply_system_model(&mut model, &contents).map_err(|e| {
            eyre!(
                "failed to parse {}: {}",
                file.display(),
                install_model::parse_error_detail(&e)
            )
        })?;
    }

    model.imported = true;
    Ok(model)
}

/// The first existing candidate document under a directory
fn find_doc(dir: &Path, nested: &str, flat: &str) -> Option<PathBuf> {
    [dir.join(nested), dir.join(flat)]
        .into_iter()
        .find(|path| path.is_file())
}

// Main entry point
#[tokio::main]
async fn main() -> Result<()> {
    setup_eyre();
    configure_tracing()?;

    let mut installer = Installer::builder()
        .add_step("storage")
        .add_step("locale")
        .add_step("timezone")
        .add_step("desktop")
        .add_step("accounts")
        .add_step("summary")
        .active_step("storage")
        .build()
        .await?;

    // Make every choice step available; summary is unlocked by the
    // frontend once the other steps have ran
    installer.make_step_available("storage")?;
    installer.make_step_available("locale")?;
    installer.make_step_available("timezone")?;
    installer.make_step_available("desktop")?;
    installer.make_step_available("accounts")?;

    let mut system = installer.system().await?;
    let info = system.get_os_info(()).await?;

    let iface = Frontend::new(installer, info.into_inner())?;
    let model = model_arg()?.unwrap_or_default();

    iface.run(model).await?;
    Ok(())
}
