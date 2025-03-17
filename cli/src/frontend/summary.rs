// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use async_trait::async_trait;
use installer::{register_step, DisplayInfo, Installer, Step, StepError};
use tracing::error;

pub struct SummaryStep {
    info: DisplayInfo,
}

impl SummaryStep {
    pub fn new() -> Self {
        Self {
            info: DisplayInfo {
                title: "Summary".to_string(),
                description: "Review the installation summary".to_string(),
                icon: None,
            },
        }
    }
}

#[async_trait]
impl Step for SummaryStep {
    fn info(&self) -> &DisplayInfo {
        &self.info
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }

    async fn run(&self, _installer: &Installer) -> Result<(), StepError> {
        error!("Summary page not yet implemented");
        Ok(())
    }
}

register_step! {
    id: "summary",
    author: "AerynOS Developers",
    description: "Review the installation summary",
    create: || Box::new(SummaryStep::new())
}
