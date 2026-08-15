use async_trait::async_trait;
use mcp_sdk::schemas::credentials::ProviderCredential;

use crate::schemas::identity::GoogleIdentity;

#[async_trait]
pub trait GoogleProvider: Send + Sync {
    async fn get_identity(&self, credential: &ProviderCredential)
        -> Result<GoogleIdentity, String>;
}
