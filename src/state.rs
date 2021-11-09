use crate::*;
use anyhow::Result;
use chrono::prelude::*;
use futures::StreamExt;
use k8s_openapi::api::apps::v1::{DeploymentSpec, DeploymentStatus};
use kube::{
    api::{Api, Informer, Object, WatchEvent},
    client::APIClient,
    config::Configuration,
};
use prometheus::{
    default_registry,
    proto::MetricFamily,
    {IntCounter, IntCounterVec, IntGauge, IntGaugeVec},
};
use std::{
    collections::BTreeMap,
    env,
    sync::{Arc, RwLock},
};

pub type Deployment = Object<DeploymentSpec, DeploymentStatus>;

/// Metrics exposed to /metrics
#[derive(Clone)]
pub struct Metrics {
    pub handled_events: IntCounter,
}

impl Metrics {
    fn new() -> Self {
        Metrics {
            handled_events: register_int_counter!("handled_events", "handled events").unwrap(),
        }
    }
}

/// In-memory state of current goings-on exposed on /
#[derive(Clone, Serialize)]
pub struct State {
    #[serde(deserialize_with = "from_ts")]
    pub last_event: DateTime<Utc>,
}

impl State {
    fn new() -> Self {
        State {
            last_event: Utc::now(),
        }
    }
}

/// User state for Actix and controller
#[derive(Clone)]
pub struct Controller {
    /// A controller settings
    config: Settings,
    /// An informer for Deployment
    info: Informer<Deployment>,
    /// In memory state
    state: Arc<RwLock<State>>,
    /// Various Prometheus metrics
    metrics: Arc<RwLock<Metrics>>,
    /// A kube client for performing cluster actions on Deployment events
    client: APIClient,
    /// A secrets storage manager
    store: Store,
}

/// Controller that wathes Deployments
impl Controller {
    async fn new(client: APIClient, config: Settings) -> Result<Self> {
        let resource = Api::v1Deployment(client.clone());
        let info = Informer::new(resource).timeout(15).init().await?;
        let metrics = Arc::new(RwLock::new(Metrics::new()));
        let state = Arc::new(RwLock::new(State::new()));
        let store = Store::new(client.clone(), config.clone()).await?;
        Ok(Controller {
            config,
            info,
            metrics,
            state,
            client,
            store,
        })
    }

    /// Metrics getter
    pub fn metrics(&self) -> Vec<MetricFamily> {
        default_registry().gather()
    }
    /// State getter
    pub fn state(&self) -> Result<State> {
        // unwrap for users because Poison errors are not great to deal with atm
        // rather just have the handler 500 above in this case
        let res = self.state.read().unwrap().clone();
        Ok(res)
    }

    /// Internal poll for internal thread
    async fn poll(&self) -> Result<()> {
        let mut deploys = self.info.poll().await?.boxed();
        while let Some(event) = deploys.next().await {
            match self.handle_event(event?).await {
                Ok(_) => {}
                Err(e) => warn!("Cannot process service: {}", e),
            };
        }
        Ok(())
    }

    /// Handle deployment events and make some things for some kinds
    async fn handle_event(&self, ev: WatchEvent<Deployment>) -> Result<()> {
        match ev {
            WatchEvent::Added(deploy) => {
                info!("Deployment {:?} added...", deploy.metadata.name);
                info!("Check deployment filtering...");

                let deploy_namespace = deploy
                    .metadata
                    .namespace
                    .clone()
                    .unwrap_or_else(|| "default".to_string());
                let deploy_name = deploy.metadata.name.clone();
                info!(
                    "Deployment: {} with namespace: {}",
                    deploy_name, &deploy_namespace
                );
                if let Some(filter) = self.config.filter.clone() {
                    if !filter.namespaces.is_empty()
                        && !filter.namespaces.contains(&deploy_namespace)
                    {
                        info!("Skip this deployment from different namespace");
                        return Err(anyhow::format_err!(
                            "Skip this deployment from different namespace"
                        ));
                    }
                }

                info!("Fetch service name...");
                let service_name = self.get_service_name(deploy.clone())?;

                let generator = rsa_generator::Generator::new(self.config.rsa.bits, service_name)?;
                self.store
                    .handle_add(deploy.metadata.namespace.clone(), generator)
                    .await?;

                if self.config.volumes.mount {
                    info!("Initialize mounter...");
                    let mounter =
                        mounter::Mounter::new(self.client.clone(), deploy, self.config.clone())
                            .await?;
                    info!("Mount...");
                    mounter.mount().await?;
                } else {
                    info!("Mount is not set... skipping...")
                }

                info!("Write to metrics...");
                self.metrics.write().unwrap().handled_events.inc();
            }
            WatchEvent::Deleted(deploy) => {
                info!("Deployment {:?} deleted...", deploy.metadata.name);

                let service_name = self.get_service_name(deploy.clone())?;
                self.store
                    .handle_delete(deploy.metadata.namespace, service_name)
                    .await?;

                self.metrics.write().unwrap().handled_events.inc();
            }
            _ => debug!("Unsupported event"),
        }

        self.state.write().unwrap().last_event = Utc::now();
        Ok(())
    }

    fn get_service_name(&self, deployment: Deployment) -> Result<String> {
        deployment
            .metadata
            .annotations
            .clone()
            .get(&self.config.annotation)
            .ok_or_else(|| {
                format!(
                    "deployment '{}' is not evrone service",
                    deployment.metadata.name.clone()
                )
            })
            .map_err(anyhow::Error::msg)
            .map(|_| deployment.metadata.name)
    }
}

/// Lifecycle initialization interface for app
///
/// This returns a `Controller` and calls `poll` on it continuously.
pub async fn init(cfg: Configuration, settings: settings::Settings) -> Result<Controller> {
    let c = Controller::new(APIClient::new(cfg), settings).await?; //for app to read
    let c2 = c.clone(); //for poll thread to write
    tokio::spawn(async move {
        loop {
            if let Err(e) = c2.poll().await {
                error!("Kube state failed to recover: {}", e);
                // rely on kube's crash loop backoff to retry sensibly:
                std::process::exit(1);
            }
        }
    });
    Ok(c)
}
