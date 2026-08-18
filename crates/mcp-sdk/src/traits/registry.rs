use serde::Serialize;

/// Declarative identity for an MCP server implementation.
///
/// Construction is intentionally owned by the application composition root,
/// because it may need to provide credentials, clients, or other services.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct McpServerRegistration {
    key: &'static str,
    name: &'static str,
    version: &'static str,
}

impl McpServerRegistration {
    pub const fn new(key: &'static str, name: &'static str, version: &'static str) -> Self {
        Self { key, name, version }
    }

    pub const fn key(self) -> &'static str {
        self.key
    }

    pub const fn name(self) -> &'static str {
        self.name
    }

    pub const fn version(self) -> &'static str {
        self.version
    }
}
