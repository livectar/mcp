use mcp_protocol::constants::MAX_JSON_PAYLOAD_BYTES;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TransportLimits {
    pub max_request_bytes: usize,
    pub max_tool_input_bytes: usize,
    pub max_tool_output_bytes: usize,
    pub max_response_bytes: usize,
    pub max_tools_per_page: usize,
    pub request_timeout: Duration,
    pub max_concurrency: usize,
}

impl Default for TransportLimits {
    fn default() -> Self {
        Self {
            max_request_bytes: MAX_JSON_PAYLOAD_BYTES,
            max_tool_input_bytes: MAX_JSON_PAYLOAD_BYTES,
            max_tool_output_bytes: MAX_JSON_PAYLOAD_BYTES,
            max_response_bytes: MAX_JSON_PAYLOAD_BYTES,
            max_tools_per_page: 50,
            request_timeout: Duration::from_secs(30),
            max_concurrency: 64,
        }
    }
}

impl TransportLimits {
    pub fn normalized(mut self) -> Self {
        self.max_request_bytes = self.max_request_bytes.max(1);
        self.max_tool_input_bytes = self.max_tool_input_bytes.max(1);
        self.max_tool_output_bytes = self.max_tool_output_bytes.max(1);
        self.max_response_bytes = self.max_response_bytes.max(1);
        self.max_tools_per_page = self.max_tools_per_page.max(1);
        self.max_concurrency = self.max_concurrency.max(1);
        self
    }
}
