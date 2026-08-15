use async_trait::async_trait;
use mcp_sdk::schemas::credentials::ProviderCredential;

use crate::schemas::identity::TelegramIdentity;

#[async_trait]
pub trait TelegramProvider: Send + Sync {
    async fn get_identity(
        &self,
        credential: &ProviderCredential,
    ) -> Result<TelegramIdentity, String>;
}
