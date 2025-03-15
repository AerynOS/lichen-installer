// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

#![allow(unused_qualifications)]

use hyper_util::rt::TokioIo;
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
