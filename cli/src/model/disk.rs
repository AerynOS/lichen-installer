// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::path::PathBuf;

use provisioning::StrategyDefinition;

/// Disk configuration
///
/// Despite requiring the plan to be executed again, it's worth realising
/// the StrategyDefinition itself is immutable, and that there is only one
/// outcome for a given set of inputs.
///
/// This ensures in future we can have a backend that is fed a plan and
/// executes it, without needing to worry about the state of the plan
/// changing underneath it.
pub struct Model {
    /// Device pool to (re-)seed the provisioner with
    pub devices: Vec<PathBuf>,

    /// Disk strategy to apply.
    pub strategy: StrategyDefinition,

    /// The disk strategy may yield multiple plans due to `find-disk`, so
    /// ensure we track the one the user picked from the plan list.
    /// The list indices are stable.
    pub plan_index: usize,
}
