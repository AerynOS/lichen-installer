// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

pub(crate) struct BuiltinStategy {
    pub name: &'static str,
    pub contents: &'static str,
}

const USE_WHOLE_DISK: BuiltinStategy = BuiltinStategy {
    name: "use_whole_disk.kdl",
    contents: include_str!("../../data/strategies/use_whole_disk.kdl"),
};

pub(crate) const ALL: &[BuiltinStategy] = &[USE_WHOLE_DISK];
