// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{env, sync::Arc};

use locales_rs::Registry;
use protocols::lichen::locales::{locales_server, GetLocaleRequest, ListLocalesResponse, Locale};
use tokio::process::Command;
use tonic::{Request, Response};
use tracing::info;

use crate::auth::AuthService;

/// System service for queries and shutdown
pub struct Service {
    _auth: Arc<AuthService>,

    // The locales registry
    registry: Registry,

    // Known locales
    locale_codes: Vec<String>,
}

/// Creates a new gRPC server instance using the default Service implementation
pub async fn service(auth: Arc<AuthService>) -> color_eyre::Result<locales_server::LocalesServer<Service>> {
    let registry = Registry::new()?;

    let output = Command::new("localectl").arg("list-locales").output().await?;
    let text = String::from_utf8(output.stdout)?;
    let locale_codes = text.lines().map(|l| l.to_string()).collect::<Vec<_>>();

    info!(num_locales = locale_codes.len(), "Loaded system locale codes");

    let current_lang = env::var("LANG").unwrap_or("en_US.UTF-8".to_string());
    let current_locale = registry.locale(&current_lang);
    if let Some(locale) = current_locale {
        info!(lang = current_lang, "Current system locale is {}", locale.display_name);
    } else {
        info!("No current system locale found");
    }

    let server = locales_server::LocalesServer::new(Service {
        _auth: auth,
        registry,
        locale_codes,
    });

    Ok(server)
}

#[tonic::async_trait]
impl locales_server::Locales for Service {
    /// Lists all available locales on the system
    async fn list_locales(&self, _request: Request<()>) -> Result<Response<ListLocalesResponse>, tonic::Status> {
        todo!()
    }

    /// Gets the locale details for a specific locale
    async fn get_locale(&self, _request: Request<GetLocaleRequest>) -> Result<Response<Locale>, tonic::Status> {
        let request = _request.into_inner();
        let locale_code = request.name;

        match self.registry.locale(&locale_code) {
            Some(locale) => {
                let conv = locale.into();
                Ok(Response::new(conv))
            }
            None => Err(tonic::Status::not_found(format!(
                "Locale code {} not found",
                locale_code
            ))),
        }
    }
}
