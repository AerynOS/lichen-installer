// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo:rerun-if-changed=locales.proto");
    println!("cargo:rerun-if-changed=osinfo.proto");
    println!("cargo:rerun-if-changed=system.proto");
    println!("cargo:rerun-if-changed=storage/disks.proto");
    println!("cargo:rerun-if-changed=storage/provisioner.proto");
    println!("cargo:rerun-if-changed=storage/types.proto");

    tonic_build::configure()
        .build_server(true)
        .compile_protos(
            &[
                "locales.proto",
                "osinfo.proto",
                "system.proto",
                "storage/disks.proto",
                "storage/provisioner.proto",
                "storage/types.proto",
            ],
            &["."],
        )
        .unwrap();
}
