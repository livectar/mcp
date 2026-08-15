use mcp_sdk::schemas::authorization::OperationName;

pub fn operation(name: &str) -> OperationName {
    OperationName::new(name).expect("test operation must be valid")
}
