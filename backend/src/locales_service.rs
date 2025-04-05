// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use protocols::lichen::locales::{locales_server, GetLocaleRequest, ListLocalesResponse, Locale};
use tonic::{Request, Response};

use crate::auth::AuthService;

/// System service for queries and shutdown
#[derive(Debug)]
pub struct Service {
    _auth: Arc<AuthService>,
}

/// Creates a new gRPC server instance using the default Service implementation
pub fn service(auth: Arc<AuthService>) -> locales_server::LocalesServer<Service> {
    locales_server::LocalesServer::new(Service { _auth: auth })
}

#[tonic::async_trait]
impl locales_server::Locales for Service {
    /// Lists all available locales on the system
    async fn list_locales(&self, _request: Request<()>) -> Result<Response<ListLocalesResponse>, tonic::Status> {
        todo!()
    }

    /// Gets the locale details for a specific locale
    async fn get_locale(&self, _request: Request<GetLocaleRequest>) -> Result<Response<Locale>, tonic::Status> {
        todo!()
    }
}
