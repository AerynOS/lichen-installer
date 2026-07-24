// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! Frontend module for disk selection during installation
//!
//! This module provides the disk selection step of the installation process,
//! allowing users to choose which disk to install AerynOS on, preview the
//! partitioning strategy, and apply it.

use crate::{CliStep, FrontendStep, install_model, selections};
use installer::{DisplayInfo, Icon, Installer, Model, StepError, register_step};
use protocols::lichen::{
    osinfo::OsInfo,
    storage::{
        disks::{Disk, ListDisksRequest},
        provisioner::{StrategyDefinition, StrategyPlan, TryStrategyRequest},
    },
};
use std::{collections::BTreeSet, env};

/// Root filesystem choices as strategy id suffixed, first entry is default
const FS_CHOICES: &[(&str, &str, &str)] = &[
    ("_xfs", "xfs", "Recommended for most users"),
    ("_f2fs", "f2fs", "Flash-friendly filesystem"),
    ("_ext4", "ext4", "The traditional Linux filesystem"),
];

pub async fn run(info: &OsInfo, installer: &Installer, model: &mut Model) -> Result<(), StepError> {
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
    let initial_idx = disks
        .disks
        .iter()
        .position(|disk| disk.device == model.storage.disk)
        .unwrap_or(0);
    let idx = cliclack::select(format!("What disk would you like to install {os_name} on?"))
        .items(&renderable_devices)
        .initial_value(initial_idx)
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

    // Look for a system model left by a previous installation on this disk
    let mut install = installer.install().await?;
    let discovered = install
        .discover_system_models(())
        .await?
        .into_inner()
        .models
        .into_iter()
        .find(|model| selected_disk.partitions.iter().any(|part| part.device == model.device));
    let mut display: Vec<usize> = Vec::new();

    viable.iter().enumerate().for_each(|(idx, (strat, _))| {
        let base = base_strategy_id(&strat.id);
        if !display.iter().any(|&seen| base_strategy_id(&viable[seen].0.id) == base) {
            display.push(idx);
        }
    });

    let mut items = display
        .iter()
        .enumerate()
        .map(|(pos, &idx)| {
            let (strat, _) = &viable[idx];
            (pos, base_strategy_id(&strat.name), strat.description.clone())
        })
        .collect::<Vec<_>>();
    let refresh_idx = viable.len();

    if discovered.is_some() {
        items.push((
            refresh_idx,
            "Refresh OS",
            "Reinstall using the settings and package selection found on the disk".to_string(),
        ));
    }

    let init_choice = if model.imported {
        display
            .iter()
            .position(|&idx| base_strategy_id(&viable[idx].0.id) == base_strategy_id(&model.storage.strategy_id))
            .unwrap_or(0)
    } else if discovered.is_some() {
        refresh_idx
    } else {
        0
    };

    let choice = cliclack::select("How should the disk be partitioned?")
        .items(&items)
        .initial_value(init_choice)
        .interact()
        .map_err(|_| StepError::UserAborted)?;

    if choice == refresh_idx
        && let Some(m) = &discovered
    {
        *model = install_model::from_kdl(&m.contents).map_err(|e| {
            StepError::Failed(format!(
                "failed to parse system model from {}: {}",
                m.device,
                install_model::parse_error_detail(&e)
            ))
        })?;
        model.imported = true;

        let mut packages: BTreeSet<String> = model.software.packages.iter().cloned().collect();
        packages.extend(selections::mandatory(&model.software.selection)?);
        model.software.packages = packages.into_iter().collect();
    }

    let (strategy, plan) = if choice == refresh_idx {
        // Partition with the strategy recorded in the discovered model,
        // falling back to the first viable strategy for this disk
        viable
            .iter()
            .find(|(strat, _)| strat.id == model.storage.strategy_id)
            .unwrap_or(&viable[0])
    } else {
        let (base, _) = &viable[display[choice]];
        let chosen_id = select_filesystem(base, &viable, &model.storage.strategy_id)?;

        viable
            .iter()
            .find(|(strat, _)| strat.id == chosen_id)
            .ok_or_else(|| StepError::Failed(format!("strategy {chosen_id} disappeared from the viable list")))?
    };

    cliclack::note(
        format!("Planned changes for {}", selected_disk.device),
        render_plan(plan),
    )
    .map_err(|_| StepError::UserAborted)?;

    model.storage.disk = selected_disk.device.clone();
    model.storage.disk_display = render_disk(selected_disk);
    model.storage.strategy_id = strategy.id.clone();
    model.storage.strategy_name = strategy.name.clone();
    model.storage.plan = Some(plan.clone());

    Ok(())
}

/// A strategy id with any filesystem-variant suffix removed
fn base_strategy_id(id: &str) -> &str {
    FS_CHOICES
        .iter()
        .find_map(|(suffix, _, _)| id.strip_suffix(suffix))
        .unwrap_or(id)
}

/// Ask which root filesystem to use
fn select_filesystem(
    representative: &StrategyDefinition,
    viable: &[(StrategyDefinition, StrategyPlan)],
    recorded_id: &str,
) -> Result<String, StepError> {
    let base = base_strategy_id(&representative.id);
    let available = FS_CHOICES
        .iter()
        .map(|(suffix, name, hint)| (format!("{base}{suffix}"), *name, *hint))
        .filter(|(id, _, _)| viable.iter().any(|(strat, _)| &strat.id == id))
        .collect::<Vec<_>>();

    // Not a filesystem-variant strategy: nothing to ask
    if available.is_empty() {
        return Ok(representative.id.clone());
    }

    // A strategy without variants has nothing to ask about
    if available.len() == 1 {
        return Ok(available[0].0.clone());
    }

    let init = available.iter().position(|(id, _, _)| id == recorded_id).unwrap_or(0);
    let items = available
        .iter()
        .enumerate()
        .map(|(idx, (_, name, hint))| (idx, name.to_string(), hint.to_string()))
        .collect::<Vec<_>>();
    let picked = cliclack::select("Which filesystem should the root partition use?")
        .items(&items)
        .initial_value(init)
        .interact()
        .map_err(|_| StepError::UserAborted)?;

    Ok(available[picked].0.clone())
}

fn render_disk(disk: &Disk) -> String {
    format!(
        "{} - {} - {}",
        disk.device,
        disk.model.as_ref().unwrap_or(&"Unknown".into()),
        disk.display_size,
    )
}

pub fn render_plan(plan: &StrategyPlan) -> String {
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
