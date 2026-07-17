// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module for disk selection during installation
//!
//! This module provides the disk selection step of the installation process,
//! allowing users to choose which disk to install AerynOS on, preview the
//! partitioning strategy, and apply it.

use crate::{CliStep, FrontendStep};
use installer::{register_step, DisplayInfo, Icon, Installer, StepError};
use protocols::lichen::{
    osinfo::OsInfo,
    storage::{
        disks::{Disk, ListDisksRequest},
        provisioner::{ApplyStrategyRequest, StrategyPlan, TryStrategyRequest},
    },
};
use std::env;

pub async fn run(info: &OsInfo, installer: &Installer) -> Result<(), StepError> {
    // Grab the list of disks. Loopback devices stay hidden unless explicitly
    // requested, which allows safe end-to-end testing against a losetup disk.
    let mut client = installer.disks().await?;
    let exclude_loopback = env::var_os("LICHEN_INCLUDE_LOOPBACK").is_none();
    let disks = client
        .list_disks(ListDisksRequest { exclude_loopback })
        .await?
        .into_inner();
    let renderable_devices = disks
        .disks
        .iter()
        .enumerate()
        .map(|(idx, dsk)| (idx, render_disk(dsk), "".to_string()))
        .collect::<Vec<_>>();
    let os_name = info
        .metadata
        .as_ref()
        .and_then(|meta| meta.identity.as_ref())
        .map(|ident| ident.display.clone())
        .unwrap_or("Unknown OS".into());
    let idx = cliclack::select(format!("What disk would you like to install {os_name} on?"))
        .items(&renderable_devices)
        .interact()
        .map_err(|_| StepError::UserAborted)?;
    let selected_disk = disks.disks.get(idx).ok_or(StepError::UserAborted)?;

    tracing::info!("Selected disk: {:?}", selected_disk.device);

    let mut provisioner = installer.provisioner().await?;
    let strategies = provisioner.list_strategies(()).await?.into_inner().strategies;

    // Keep only the strategies that yield at least one plan for the chosen disk
    let mut viable = Vec::new();

    for strat in &strategies {
        let plans = provisioner
            .try_strategy(TryStrategyRequest {
                strategy: strat.id.clone(),
                disks: vec![selected_disk.device.clone()],
            })
            .await?
            .into_inner()
            .plans;

        if let Some(plan) = plans.into_iter().next() {
            viable.push((strat.clone(), plan));
        }
    }

    if viable.is_empty() {
        return Err(StepError::Failed(format!(
            "No partitioning strategy is applicable to {}",
            selected_disk.device
        )));
    }

    let items = viable
        .iter()
        .enumerate()
        .map(|(idx, (strat, _))| (idx, strat.name.clone(), strat.description.clone()))
        .collect::<Vec<_>>();
    let choice = cliclack::select("How should the disk be partitioned?")
        .items(&items)
        .interact()
        .map_err(|_| StepError::UserAborted)?;
    let (strategy, plan) = &viable[choice];

    cliclack::note(
        format!("Planned changes for {}", selected_disk.device),
        render_plan(plan),
    )
    .map_err(|_| StepError::UserAborted)?;

    let confirmed = cliclack::confirm(format!(
        "Erase {} and apply `{}`? ALL DATA ON THIS DISK WILL BE DESTROYED.",
        selected_disk.device, strategy.name,
    ))
    .initial_value(false)
    .interact()
    .map_err(|_| StepError::UserAborted)?;

    if !confirmed {
        return Err(StepError::UserAborted);
    }

    let applied = provisioner
        .apply_strategy(ApplyStrategyRequest {
            strategy: strategy.id.clone(),
            disks: vec![selected_disk.device.clone()],
        })
        .await?
        .into_inner();

    if let Some(plan) = applied.plan {
        let mounts = plan
            .role_mounts
            .iter()
            .map(|role_mount| format!("  {} on {}", role_mount.device, role_mount.mountpoint))
            .collect::<Vec<_>>()
            .join("\n");

        let _ = cliclack::log::success(format!("Disk partitioned and formatted:\n{mounts}"));
    }

    Ok(())
}

fn render_disk(disk: &Disk) -> String {
    format!(
        "{} - {} - {}",
        disk.device,
        disk.model.as_ref().unwrap_or(&"Unknown".into()),
        disk.display_size,
    )
}

fn render_plan(plan: &StrategyPlan) -> String {
    let mut out = String::new();

    plan.disk_plans
        .iter()
        .for_each(|disk_plan| out.push_str(&format!("{}:\n{}\n", disk_plan.device, disk_plan.description)));
    if !plan.filesystems.is_empty() {
        out.push_str("\nFilesystems:\n");

        plan.filesystems.iter().for_each(|pf| {
            if let Some(fs) = &pf.filesystem {
                out.push_str(&format!(
                    "  {} -> {} ({})\n",
                    pf.device,
                    fs.filesystem_type,
                    fs.label.as_deref().unwrap_or("no label"),
                ));
            }
        });
    }

    if !plan.role_mounts.is_empty() {
        out.push_str("\nMounts:\n");
        plan.role_mounts.iter().for_each(|role_mount| {
            out.push_str(&format!("  {} <- {}\n", role_mount.mountpoint, role_mount.device));
        });
    }

    out
}

register_step! {
    id: "storage",
    author: "AerynOS Developers",
    description: "Select the disk to install on",
    create: || Box::new(
        CliStep {
            info: DisplayInfo {
                title: "Configure storage".to_string(),
                description: "Select the disk to install on".to_string(),
                icon: Some(Icon::Emoji("💾".to_string())),
            },
            step: FrontendStep::Storage,
        }
    )
}
