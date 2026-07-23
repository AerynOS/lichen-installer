// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use protocols::lichen::storage::provisioner::StrategyPlan;

/// Storage and partitioning installation settings
#[derive(Debug, Default)]
pub struct Model {
    /// Target disk path in /dev
    pub disk: String,
    /// Human readable description of the target disk
    pub disk_display: String,
    /// Identifier of the chosen provisioning strategy
    pub strategy_id: String,
    /// Display name of the chosen provisioning strategy
    pub strategy_name: String,
    /// The partitioning plan computed for the chosen disk and strategy
    pub plan: Option<StrategyPlan>,
}
