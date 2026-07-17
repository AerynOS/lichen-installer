// SPDX-FileCopyrightText: Copyright © 2025 Serpent OS Developers
// SPDX-FileCopyrightText: Copyright © 2025 AerynOS Developers
//
// SPDX-License-Identifier: MPL-2.0

use crate::{auth::AuthService, builtin_strategies, plans};
use disks::BlockDevice;
use lichen_macros::authorized;
use protocols::lichen::storage::provisioner::{
    self,
    provisioner_server::{self, ProvisionerServer},
    ApplyStrategyRequest, ApplyStrategyResponse, ListStrategiesResponse, TryStrategyRequest, TryStrategyResponse,
};
use provisioning::{Parser, StrategyDefinition};
use std::{collections::HashMap, path::Path, sync::Arc};
use tonic::{Request, Response, Status};
use tracing::{debug, info, trace};

#[derive(Debug)]
pub struct Service {
    auth: Arc<AuthService>,
    builtin_strategies: HashMap<String, StrategyDefinition>,
}

/// Creates a new gRPC server instance using the default Service implementation
pub async fn service(auth: Arc<AuthService>) -> color_eyre::Result<ProvisionerServer<Service>> {
    let mut inner = Service {
        auth: auth.clone(),
        builtin_strategies: HashMap::new(),
    };

    // Load builtin strategies
    for builtin in builtin_strategies::ALL {
        debug!("Loading builtin strategy: {}", builtin.name);
        let parser = Parser::new(builtin.name, builtin.contents)?;
        let n_strats = parser.strategies.len();

        for strat in parser.strategies {
            info!(
                filename = builtin.name,
                strategies = n_strats,
                "Loaded strategy: {}",
                strat.name,
            );
            inner.builtin_strategies.insert(strat.name.clone(), strat);
        }
    }

    let server = ProvisionerServer::new(inner);

    Ok(server)
}

impl Service {
    /// Discover block devices and select exactly the requested /dev paths
    fn selected_devices(&self, requested: &[String]) -> Result<Vec<BlockDevice>, Status> {
        if requested.is_empty() {
            return Err(Status::invalid_argument("no disks provided"));
        }

        let mut devices = BlockDevice::discover()?;
        devices.retain(|dev| requested.iter().any(|path| Path::new(path) == dev.device()));

        if devices.len() != requested.len() {
            return Err(Status::not_found("one or more requested disks were not found"));
        }

        Ok(devices)
    }
}

#[tonic::async_trait]
impl provisioner_server::Provisioner for Service {
    async fn list_strategies(&self, _request: Request<()>) -> Result<Response<ListStrategiesResponse>, Status> {
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

        let response = ListStrategiesResponse { strategies };

        Ok(Response::new(response))
    }

    #[authorized("com.aerynos.lichen.provisioner.try")]
    async fn try_strategy(
        &self,
        request: Request<TryStrategyRequest>,
    ) -> Result<Response<TryStrategyResponse>, tonic::Status> {
        let req = request.into_inner();

        trace!(strategy = req.strategy, "Trying provisioning strategy");

        if !self.builtin_strategies.contains_key(&req.strategy) {
            return Err(Status::not_found(format!("unknown strategy: {}", req.strategy)));
        }

        let devices = self.selected_devices(&req.disks)?;
        let plans = plans::try_strategy(&self.builtin_strategies, &req.strategy, &devices);

        Ok(Response::new(TryStrategyResponse { plans }))
    }

    #[authorized("com.aerynos.lichen.provisioner.apply")]
    async fn apply_strategy(
        &self,
        request: Request<ApplyStrategyRequest>,
    ) -> Result<Response<ApplyStrategyResponse>, tonic::Status> {
        let req = request.into_inner();

        info!(
            strategy = req.strategy,
            disks = ?req.disks,
            "Applying provisioning strategy (destructive)"
        );

        if !self.builtin_strategies.contains_key(&req.strategy) {
            return Err(Status::not_found(format!("unknown strategy: {}", req.strategy)));
        }

        let plan = tokio::task::block_in_place(|| {
            let devices = self.selected_devices(&req.disks)?;
            plans::apply_strategy(&self.builtin_strategies, &req.strategy, &devices)
        })?;

        Ok(Response::new(ApplyStrategyResponse { plan: Some(plan) }))
    }
}
