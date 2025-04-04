// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use nix::libc::gid_t;
use tokio::net::unix::{pid_t, uid_t};
use tonic::{transport::server::UdsConnectInfo, Request, Status};

#[derive(Debug, Clone)]
pub enum AuthInfo {
    /// Unix domain socket
    Unix { uid: uid_t, gid: gid_t, pid: Option<pid_t> },
}

/// Intercept to install our own specific helper type for PEERCRED
pub fn uds_interceptor(mut request: Request<()>) -> Result<Request<()>, Status> {
    let uds_creds = request.extensions().get::<UdsConnectInfo>();

    if let Some(peer_creds) = uds_creds.as_ref().and_then(|u| u.peer_cred) {
        let auth = AuthInfo::Unix {
            uid: peer_creds.uid(),
            gid: peer_creds.gid(),
            pid: peer_creds.pid(),
        };
        request.extensions_mut().insert(auth);
        Ok(request)
    } else {
        Err(Status::unauthenticated(
            "client socket did not share SO_PEERCRED, refusing connection",
        ))
    }
}
