// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::sync::Arc;

use protocols::lichen::storage::provisioner::{
    self,
    provisioner_server::{self, ProvisionerServer},
};
use tonic::{Request, Response};
use tracing::trace;

use crate::auth::AuthService;

/// Provisioning service for disk strategy
pub struct Service {
    _auth: Arc<AuthService>,
}

/// Creates a new gRPC server instance using the default Service implementation
pub async fn service(auth: Arc<AuthService>) -> color_eyre::Result<ProvisionerServer<Service>> {
    let server = ProvisionerServer::new(Service { _auth: auth });

    Ok(server)
}

#[tonic::async_trait]
impl provisioner_server::Provisioner for Service {
    async fn list_strategies(
        &self,
        _request: Request<()>,
    ) -> Result<Response<provisioner::ListStrategiesResponse>, tonic::Status> {
        trace!("Listing available provisioning strategies");
        let response = provisioner::ListStrategiesResponse { strategies: vec![] };
        Ok(Response::new(response))
    }

    async fn try_strategy(
        &self,
        _request: Request<provisioner::TryStrategyRequest>,
    ) -> Result<Response<provisioner::TryStrategyResponse>, tonic::Status> {
        trace!("Trying provisioning strategy");
        let response = provisioner::TryStrategyResponse {};
        Ok(Response::new(response))
    }
}
