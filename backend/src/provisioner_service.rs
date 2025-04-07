// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use std::{collections::HashMap, sync::Arc};

use protocols::lichen::storage::provisioner::{
    self,
    provisioner_server::{self, ProvisionerServer},
};
use tonic::{Request, Response};
use tracing::{debug, info, trace};

use crate::{auth::AuthService, builtin_strategies};

#[derive(Debug)]
pub struct Service {
    _auth: Arc<AuthService>,
    builtin_strategies: HashMap<String, provisioning::StrategyDefinition>,
}

/// Creates a new gRPC server instance using the default Service implementation
pub async fn service(auth: Arc<AuthService>) -> color_eyre::Result<ProvisionerServer<Service>> {
    let mut inner = Service {
        _auth: auth.clone(),
        builtin_strategies: HashMap::new(),
    };

    // Load builtin strategies
    for b in builtin_strategies::ALL {
        debug!("Loading builtin strategy: {}", b.name);
        let parser = provisioning::Parser::new(b.name, b.contents)?;
        let n_strategies = parser.strategies.len();
        for strategy in parser.strategies {
            info!(
                filename = b.name,
                strategies = n_strategies,
                "Loaded strategy: {}",
                strategy.name
            );
            inner.builtin_strategies.insert(strategy.name.clone(), strategy);
        }
    }

    let server = ProvisionerServer::new(inner);

    Ok(server)
}

#[tonic::async_trait]
impl provisioner_server::Provisioner for Service {
    async fn list_strategies(
        &self,
        _request: Request<()>,
    ) -> Result<Response<provisioner::ListStrategiesResponse>, tonic::Status> {
        trace!("Listing available provisioning strategies");
        let strategies = self
            .builtin_strategies
            .iter()
            .map(|(name, strategy)| provisioner::StrategyDefinition {
                id: name.clone(),
                name: strategy.name.clone(),
                description: strategy.summary.clone(),
                inherits: strategy.inherits.clone(),
            })
            .collect();
        let response = provisioner::ListStrategiesResponse { strategies };
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
