// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use installer::{register_step, DisplayInfo, Installer, StepError};
use tracing::error;

use crate::{CliStep, FrontendStep};

pub async fn run_summary(_installer: &Installer) -> Result<(), StepError> {
    error!("Summary step not implemented");
    Ok(())
}

register_step! {
    id: "summary",
    author: "AerynOS Developers",
    description: "Review the installation summary",
    create: || Box::new(CliStep { info: DisplayInfo {
        title: "Summary".to_string(),
        description: "Review the installation summary".to_string(),
        icon: None,
    }, step: FrontendStep::Summary })
}
