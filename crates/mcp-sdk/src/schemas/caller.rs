#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallerContext {
    pub tenant_id: String,
    pub subject_id: String,
    pub installation_id: Option<String>,
    pub connection_id: Option<String>,
}
