// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

#![allow(unused_qualifications)]

pub mod privileged;

#[cfg(feature = "backend-utils")]
pub mod backend_utils;

use std::{path::Path, sync::Arc};

use hyper_util::rt::TokioIo;
use privileged::{PkexecExecutor, ServiceConnection};
use thiserror::Error;
use tokio::net::UnixStream;
use tonic::transport::{Channel, Endpoint, Uri};
use tower::service_fn;

pub mod lichen {
    pub mod locales {
        tonic::include_proto!("lichen.locales");
    }
    pub mod osinfo {
        tonic::include_proto!("lichen.osinfo");
    }
    pub mod storage {
        pub mod disks {
            tonic::include_proto!("lichen.storage.disks");
        }
        pub mod strategy {
            tonic::include_proto!("lichen.storage.strategy");
        }
    }
    pub mod system {
        tonic::include_proto!("lichen.system");
    }
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
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

pub fn create_service_connection(executable: impl AsRef<Path>) -> Result<Arc<ServiceConnection>, Error> {
    let path = executable.as_ref().to_string_lossy().to_string();
    let connection = Arc::new(
        ServiceConnection::new::<PkexecExecutor>(&path, &[])
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
    );
    connection.socket.set_nonblocking(true)?;
    Ok(connection)
}

pub async fn service_connection_to_channel(connection: Arc<ServiceConnection>, path: String) -> Result<Channel, Error> {
    let encoded_uri = Uri::builder()
        .scheme("http")
        .authority("localhost:50051")
        .path_and_query(path)
        .build()?;

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

    Ok(channel)
}
