use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleErrorEnvelope {
    pub error: GoogleErrorBody,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleErrorBody {
    pub code: Option<u16>,
    pub message: Option<String>,
    pub status: Option<String>,
    #[serde(default)]
    pub errors: Vec<GoogleErrorDetail>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GoogleErrorDetail {
    pub reason: Option<String>,
    pub message: Option<String>,
}
