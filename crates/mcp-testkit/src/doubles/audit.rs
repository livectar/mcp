use async_trait::async_trait;
use mcp_sdk::{errors::HostError, schemas::audit::AuditEvent, traits::host::AuditSink};
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct RecordingAuditSink {
    events: Arc<Mutex<Vec<AuditEvent>>>,
}

impl RecordingAuditSink {
    pub fn events(&self) -> Vec<AuditEvent> {
        self.events.lock().expect("audit mutex poisoned").clone()
    }
}

#[async_trait]
impl AuditSink for RecordingAuditSink {
    async fn record(&self, event: AuditEvent) -> Result<(), HostError> {
        self.events
            .lock()
            .map_err(|_| HostError::Service("audit mutex poisoned".to_string()))?
            .push(event);
        Ok(())
    }
}
