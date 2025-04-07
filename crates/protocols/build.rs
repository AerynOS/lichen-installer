// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo:rerun-if-changed=disks.proto");
    println!("cargo:rerun-if-changed=locales.proto");
    println!("cargo:rerun-if-changed=osinfo.proto");
    println!("cargo:rerun-if-changed=system.proto");
    println!("cargo:rerun-if-changed=storage/strategy.proto");

    tonic_build::configure()
        .build_server(true)
        .compile_protos(
            &[
                "disks.proto",
                "locales.proto",
                "osinfo.proto",
                "system.proto",
                "storage/strategy.proto",
            ],
            &["."],
        )
        .unwrap();
}
