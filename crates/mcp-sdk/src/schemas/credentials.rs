use crate::errors::HostError;
use serde::Serialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialFieldType {
    Secret,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CredentialField {
    pub key: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub input_type: CredentialFieldType,
    pub required: bool,
}

impl CredentialField {
    pub const fn secret(key: &'static str, label: &'static str, description: &'static str) -> Self {
        Self {
            key,
            label,
            description,
            input_type: CredentialFieldType::Secret,
            required: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CredentialRequirements {
    pub provider: &'static str,
    pub secret_field: &'static str,
    pub fields: &'static [CredentialField],
}

impl CredentialRequirements {
    pub const fn new(
        provider: &'static str,
        secret_field: &'static str,
        fields: &'static [CredentialField],
    ) -> Self {
        Self {
            provider,
            secret_field,
            fields,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderName(String);

impl ProviderName {
    pub fn new(value: impl Into<String>) -> Result<Self, HostError> {
        let value = value.into();
        if value.trim().is_empty() || value.len() > 128 {
            return Err(HostError::InvalidRequest(
                "provider name must be between 1 and 128 characters".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CredentialRequest {
    pub provider: ProviderName,
    pub purpose: String,
}

#[derive(Clone, PartialEq, Eq)]
pub struct ProviderCredential(String);

impl ProviderCredential {
    pub fn new(secret: impl Into<String>) -> Result<Self, HostError> {
        let secret = secret.into();
        if secret.is_empty() {
            return Err(HostError::CredentialUnavailable);
        }
        Ok(Self(secret))
    }

    pub fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ProviderCredential {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProviderCredential(REDACTED)")
    }
}
