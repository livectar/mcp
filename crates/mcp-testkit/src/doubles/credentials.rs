use async_trait::async_trait;
use mcp_sdk::{
    errors::HostError,
    schemas::{
        caller::CallerContext,
        credentials::{CredentialRequest, ProviderCredential, ProviderName},
    },
    traits::host::CredentialResolver,
};

pub struct NoCredentials;

#[async_trait]
impl CredentialResolver for NoCredentials {
    async fn resolve(
        &self,
        _caller: &CallerContext,
        _request: &CredentialRequest,
    ) -> Result<ProviderCredential, HostError> {
        Err(HostError::CredentialUnavailable)
    }
}

pub struct StaticCredentialResolver {
    pub provider: ProviderName,
    pub credential: ProviderCredential,
}

#[async_trait]
impl CredentialResolver for StaticCredentialResolver {
    async fn resolve(
        &self,
        _caller: &CallerContext,
        request: &CredentialRequest,
    ) -> Result<ProviderCredential, HostError> {
        if request.provider == self.provider {
            Ok(self.credential.clone())
        } else {
            Err(HostError::CredentialUnavailable)
        }
    }
}
