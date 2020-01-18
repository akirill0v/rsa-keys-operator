use anyhow::Result;
use k8s_openapi::{
    api::core::v1::PodSpec,
    serde_json,
};
use kube::{
    api::{ Api, PatchParams },
    client::APIClient,
};
use serde_json::{
    json, value::Value,
};
use crate::{
    settings::Settings,
    state::Deployment,
    utils,
};

#[derive(Clone)]
pub struct Mounter {
    client: APIClient,
    deployment: Deployment,
    settings: Settings,
}

/// Mounter creates volumes in deployment
/// and restart pods after patching
impl Mounter {
    pub async fn new(client: APIClient, deployment: Deployment, settings: Settings) -> Result<Self> {
        Ok(Self { client, deployment, settings })
    }

    pub async fn mount(&self) -> Result<()> {
        info!("Mount volumes to deploy: {:?}", self.deployment.metadata.name);
        let volumes_patch = self.make_patch().await?;
        let container_volumes = self.make_containers_patch(self.deployment.clone().spec.template.spec).await?;

        let patch = json!({
            "spec": {
                "template": {
                    "spec": {
                        "containers": container_volumes,
                        "volumes": volumes_patch.get("volumes"),
                    },
                }
            }
        });

        info!("Applyed patch: {}", patch);

        let client = Api::v1Deployment(self.client.clone())
            .within(&self.deployment.clone().metadata.namespace.unwrap_or_else(|| "default".into()));

        client.patch(
            &self.deployment.metadata.name,
            &PatchParams::default(),
            serde_json::to_vec(&patch)?,
        ).await?;
        Ok(())
    }

    async fn make_containers_patch(&self, pod_spec: Option<PodSpec>) -> Result<Value> {
        let containers = pod_spec
            .map(|s| s.containers)
            .ok_or_else(|| format!("Missing containers for deployment '{}'", self.deployment.metadata.name))
            .map_err(anyhow::Error::msg)?;
        info!("Containers for deployment: {:?}  -- {:?}", self.deployment.metadata.name, containers);

        let containers_with_volumes: Vec<Value> = containers.into_iter().map(|c| {
            json!({
                "name": c.name,
                "image": c.image,
                "volumeMounts": [{
                    "name": utils::secret_name(self.deployment.metadata.name.clone()),
                    "mountPath": self.settings.volumes.private.path,
                },{
                    "name": self.settings.secrets.public_name,
                    "mountPath": self.settings.volumes.public.path,
                }],
            })
        }).collect();

        Ok(containers_with_volumes.into())
    }

    async fn make_patch(&self) -> Result<Value> {
        let patch = json!({
            "volumes": [{
                    "name": utils::secret_name(self.deployment.metadata.name.clone()),
                    "secret": {
                        "secretName": utils::secret_name(self.deployment.metadata.name.clone()),
                    },
            }, {
                "name": self.settings.secrets.public_name.clone(),
                "secret": {
                    "secretName": self.settings.secrets.public_name,
                },
            }
            ],
        });
        Ok(patch)
    }
}
