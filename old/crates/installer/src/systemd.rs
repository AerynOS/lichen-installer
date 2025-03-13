// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
//
// SPDX-License-Identifier: MPL-2.0

//! systemd helpers

use std::{io, process::Command, string::FromUtf8Error};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("io: {0}")]
    IO(#[from] io::Error),

    #[error("utf8 decoding: {0}")]
    Utf8(#[from] FromUtf8Error),
}

/// List all locales according to localectl
pub fn localectl_list_locales() -> Result<Vec<String>, Error> {
    let output = Command::new("localectl").arg("list-locales").output()?;
    let text = String::from_utf8(output.stdout)?;
    Ok(text.lines().map(|l| l.to_string()).collect::<Vec<_>>())
}
