// SPDX-FileCopyrightText: Copyright © 2026 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! btrfs specific install handling: the default @/@home subvolume layout.

use super::{blkid, run};
use crate::install_service::ResolvedMount;
use std::{path::Path, process::Command};
use tonic::Status;

pub(super) const ROOT_SUBVOL: &str = "@";
pub(super) const HOME_SUBVOL: &str = "@home";

/// Rewrite a btrfs root into the default @/@home layout: the root ("/") mount
/// moves onto @ and a /home mount on @home is derived from the same device.
/// No-op unless the root mount is btrfs, non-btrfs installs pass through untouched.
pub(super) fn expand_subvolumes(mounts: &mut Vec<ResolvedMount>) {
    let Some(root) = mounts
        .iter_mut()
        .find(|mount| mount.mountpoint == "/" && mount.fstype == "btrfs")
    else {
        return;
    };

    root.subvol = Some(ROOT_SUBVOL.to_string());
    let device = root.device.clone();
    let fstype = root.fstype.clone();

    // Only derive /home if dedicated /home mount isn't already present
    if !mounts.iter().any(|mount| mount.mountpoint == "/home") {
        mounts.push(ResolvedMount {
            device,
            mountpoint: "/home".to_string(),
            fstype,
            subvol: Some(HOME_SUBVOL.to_string()),
        });
    }
}

/// True if the device holds a btrfs filesystem.
pub(super) fn is_btrfs(device: &str) -> Result<bool, Status> {
    Ok(blkid(device, "TYPE")? == "btrfs")
}

/// Mount a freshly-formatted btrfs root, create the @ and @home subvolumes,
/// then unmount so the real subvolume mounts can take over.
pub(super) fn create_subvolumes(target: &Path, device: &str) -> Result<(), Status> {
    run(Command::new("mount").args(["-o", "subvolid=5"]).arg(device).arg(target))?;

    let result = (|| -> Result<(), Status> {
        for subvol in [ROOT_SUBVOL, HOME_SUBVOL] {
            let path = target.join(subvol);
            if !path.exists() {
                run(Command::new("btrfs").args(["subvolume", "create"]).arg(path))?;
            }
        }
        run(Command::new("btrfs")
            .args(["subvolume", "set-default"])
            .arg(target.join(ROOT_SUBVOL)))?;
        Ok(())
    })();

    let _ = run(Command::new("umount").arg(target));
    result
}
