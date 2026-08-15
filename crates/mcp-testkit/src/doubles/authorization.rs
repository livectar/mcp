use async_trait::async_trait;
use mcp_sdk::{
    errors::HostError,
    schemas::{authorization::AuthorizationRequest, caller::CallerContext},
    traits::host::Authorization,
};

pub struct AllowAllAuthorization;

#[async_trait]
impl Authorization for AllowAllAuthorization {
    async fn authorize(
        &self,
        _caller: &CallerContext,
        _request: &AuthorizationRequest,
    ) -> Result<(), HostError> {
        Ok(())
    }
}
