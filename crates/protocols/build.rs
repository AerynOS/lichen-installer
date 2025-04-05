// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

fn main() {
    println!("cargo:rerun-if-changed=disks.proto");
    println!("cargo:rerun-if-changed=locales.proto");
    println!("cargo:rerun-if-changed=osinfo.proto");
    println!("cargo:rerun-if-changed=system.proto");

    tonic_build::configure()
        .build_server(true)
        .compile_protos(
            &["disks.proto", "locales.proto", "osinfo.proto", "system.proto"],
            &["."],
        )
        .unwrap();
}
