// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo:rerun-if-changed=disks.proto");
    println!("cargo:rerun-if-changed=backend.proto");
    tonic_build::compile_protos("disks.proto").unwrap();
    tonic_build::compile_protos("backend.proto").unwrap();
}
