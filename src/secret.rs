use anyhow::Result;
use base64::encode;
use kube::{
    api::{v1Secret, Api, DeleteParams, PatchParams, PostParams},
    client::APIClient,
};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Clone)]
pub struct RsaSecret {
    /// A kube client for performing cluster actions
    api: Api<v1Secret>,

    /// Name of the secret
    name: String,

    fields: BTreeMap<String, String>,
}

/// Implements RSA secret management in Kubernetes cluster
impl RsaSecret {
    pub async fn new(client: APIClient, name: String, namespace: Option<String>) -> Result<Self> {
        Ok(RsaSecret {
            api: Api::v1Secret(client).within(&namespace.unwrap_or_else(|| "default".into())),
            name,
            fields: BTreeMap::new(),
        })
    }

    /// Set filed named `name` value from `value`
    /// It's overwrite existing value
    pub async fn add_field(&mut self, name: &str, value: &str) -> Result<&mut Self> {
        self.fields
            .entry(name.into())
            .or_insert_with(|| encode(&value));
        Ok(self)
    }

    /// Retrive real secret from Kubernetes
    pub async fn get(&self) -> Result<v1Secret> {
        self.api.get(&self.name).await.map_err(|e| e.into())
    }

    /// Update secret fields...
    pub async fn update(&self) -> Result<&Self> {
        if self.get().await.is_err() {
            let _ = self.create().await?;
        }

        let patch = json!({
            "data": serde_json::to_value(self.fields.clone())?,
        });

        self.api
            .patch(
                &self.name,
                &PatchParams::default(),
                serde_json::to_vec(&patch)?,
            )
            .await?;

        Ok(self)
    }

    /// Create real Kubernetes secret
    pub async fn create(&self) -> Result<&Self> {
        warn!("Create new secret: {}", self.name);
        let p = json!({
            "apiVersion": "v1",
            "kind": "Secret",
            "metadata": {
                "name": self.name,
            },
            "type": "Opaque",
            "data": {},
        });

        self.api
            .create(&PostParams::default(), serde_json::to_vec(&p)?)
            .await?;

        Ok(self)
    }

    /// Clean fields in read Kubernetes secret
    pub async fn clean(&self, fields: Vec<String>) -> Result<&Self> {
        info!("Clean secrets for service {}", self.name);
        let mut data = self.api.get(&self.name).await.map(|s| s.data)?;
        for field in fields.iter() {
            info!("Remove field '{}' in '{}'", field, &self.name);
            data.remove(field);
        }

        if data.is_empty() {
            // NO one key contains - delete secret and return
            warn!("Secret {} is empty... remove it now", self.name);
            self.api
                .delete(&self.name, &DeleteParams::default())
                .await?;
            return Ok(self);
        }

        let p = json!({
            "apiVersion": "v1",
            "kind": "Secret",
            "metadata": {
                "name": self.name,
            },
            "type": "Opaque",
            "data": serde_json::to_value(data)?,
        });

        self.api
            .replace(&self.name, &PostParams::default(), serde_json::to_vec(&p)?)
            .await?;
        Ok(self)
    }
}
