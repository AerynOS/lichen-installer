// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo:rerun-if-changed=disks.proto");
    tonic_build::compile_protos("disks.proto").unwrap();
}
