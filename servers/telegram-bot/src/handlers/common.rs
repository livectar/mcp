use mcp_protocol::schemas::{
    json_payload::JsonPayload,
    tools::{CallToolResult, ContentBlock},
};
use mcp_sdk::{
    errors::{HostError, ServerError},
    schemas::{
        authorization::{ApprovalDecision, AuthorizationRequest, OperationName},
        context::RequestContext,
        credentials::{ProviderCredential, ProviderName},
    },
};
use serde::{de::DeserializeOwned, Serialize};

pub(crate) async fn authorize_and_credential(
    context: &RequestContext,
    tool_name: &'static str,
    purpose: &'static str,
) -> Result<ProviderCredential, ServerError> {
    let operation = AuthorizationRequest {
        operation: OperationName::new(tool_name)?,
        tool_name: tool_name.to_string(),
    };
    context.authorize(&operation).await?;
    if context.approval(&operation).await? == ApprovalDecision::Denied {
        return Err(HostError::ApprovalDenied.into());
    }
    context
        .credential(ProviderName::new("telegram-bot")?, purpose)
        .await
        .map_err(ServerError::from)
}

pub(crate) fn decode_arguments<T>(arguments: Option<JsonPayload>) -> Result<T, ServerError>
where
    T: DeserializeOwned,
{
    arguments
        .ok_or_else(|| ServerError::InvalidArguments("tool arguments are required".to_string()))?
        .decode::<T>()
        .map_err(|error| ServerError::InvalidArguments(error.to_string()))
}

pub(crate) fn success<T: Serialize>(
    summary: String,
    result: &T,
) -> Result<CallToolResult, ServerError> {
    let structured_content = JsonPayload::from_serializable(result)
        .map_err(|error| ServerError::Protocol(error.to_string()))?;
    Ok(CallToolResult {
        content: vec![ContentBlock::Text { text: summary }],
        structured_content: Some(structured_content),
        is_error: false,
    })
}

pub(crate) fn provider_error(error: impl std::fmt::Display) -> ServerError {
    ServerError::Provider(error.to_string())
}
