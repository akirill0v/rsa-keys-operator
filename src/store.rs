use anyhow::Result;
use kube::{
    api::{v1Secret, Api, PatchParams, PostParams, Reflector},
    client::APIClient,
};
use std::str::from_utf8;

use k8s_openapi::serde_json;
use serde_json::json;

use crate::{rsa_generator::Generator, secret::RsaSecret, settings::Settings, utils};

/// Storage to manage kubernetes secrets
#[derive(Clone)]
pub struct Store {
    /// A kube client for performing cluster actions
    client: APIClient,
    config: Settings,
}

/// Implements Store methods for manage kubernetes secrets
impl Store {
    pub async fn new(client: APIClient, config: Settings) -> Result<Self> {
        Ok(Store { client, config })
    }

    /// Update existing secret with new rsa fields
    pub async fn handle_add(&self, namespace: Option<String>, generator: Generator) -> Result<()> {
        info!("Add token fields for <{}>", &generator.name);

        let mut private_secret = RsaSecret::new(
            self.client.clone(),
            utils::secret_name(generator.name.clone()),
            namespace,
        )
        .await?;

        private_secret
            .add_field("private.pem", from_utf8(&generator.private_key)?)
            .await?
            .update()
            .await?;

        let key_name = format!("{}.pem", generator.name);

        for ns in self.config.secrets.public_namespaces.iter() {
            let mut public_secret = RsaSecret::new(
                self.client.clone(),
                self.config.secrets.public_name.clone(),
                Some(ns.into()),
            )
            .await?;

            public_secret
                .add_field(&key_name, from_utf8(&generator.public_key)?)
                .await?
                .update()
                .await?;
        }

        Ok(())
    }

    /// Remove rsa fields from secret...
    pub async fn handle_delete(
        &self,
        namespace: Option<String>,
        service_name: String,
    ) -> Result<()> {
        info!("Delete token fields for <{}>", service_name);

        let key_name = format!("{}.pem", service_name);

        for ns in self.config.secrets.public_namespaces.iter() {
            let public_secret = RsaSecret::new(
                self.client.clone(),
                self.config.secrets.public_name.clone(),
                Some(ns.into()),
            )
            .await?;

            public_secret.clean(vec![key_name.clone()]).await?;
        }

        let private_secret = RsaSecret::new(
            self.client.clone(),
            utils::secret_name(service_name.clone()),
            namespace,
        )
        .await?;

        private_secret.clean(vec!["private.pem".into()]).await?;

        Ok(())
    }
}
