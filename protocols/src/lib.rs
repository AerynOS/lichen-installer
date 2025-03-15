// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

#![allow(unused_qualifications)]

pub mod privileged;

use std::{path::Path, sync::Arc};

use hyper_util::rt::TokioIo;
use nix::unistd::Pid;
use privileged::{PkexecExecutor, ServiceConnection};
use thiserror::Error;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

pub mod proto_disks {
    tonic::include_proto!("disks");
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Tonic error: {0}")]
    Tonic(#[from] tonic::transport::Error),
    #[error("Uri error: {0}")]
    Uri(#[from] http::Error),
    #[error("Privileged error: {0}")]
    Privileged(#[from] privileged::Error),
}

/// Create a new channel to the Unix domain socket server.
pub async fn unix_channel(whence: &str) -> Result<Channel, Error> {
    let encoded_uri = Uri::builder()
        .scheme("http")
        .authority("localhost:50051")
        .path_and_query(whence)
        .build()?;
    let channel = Endpoint::from(encoded_uri)
        .connect_with_connector(service_fn(move |uri: Uri| async move {
            let stream = UnixStream::connect(uri.path()).await?;
            let res = TokioIo::new(stream);
            Ok::<_, std::io::Error>(res)
        }))
        .await?;
    Ok(channel)
}

pub async fn privileged_channel(executable: impl AsRef<Path>) -> Result<(Pid, Channel), Error> {
    let path = executable.as_ref().to_string_lossy().to_string();
    let encoded_uri = Uri::builder()
        .scheme("http")
        .authority("localhost:50051")
        .path_and_query(path.clone())
        .build()?;

    // Keep the whole ServiceConnection alive
    let connection = Arc::new(
        ServiceConnection::new::<PkexecExecutor>(&path, &[])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
    );
    connection.socket.set_nonblocking(true)?;
    let pid = connection._child;

    let connection_clone = Arc::clone(&connection);
    let channel = Endpoint::from(encoded_uri)
        .connect_with_connector(service_fn(move |_: Uri| {
            let connection = Arc::clone(&connection_clone);
            async move {
                let stream = UnixStream::from_std(connection.socket.try_clone()?)?;
                let res = TokioIo::new(stream);
                Ok::<_, std::io::Error>(res)
            }
        }))
        .await?;

    Ok((pid, channel))
}
