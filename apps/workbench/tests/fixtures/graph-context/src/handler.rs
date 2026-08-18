// HANDWRITE-BEGIN gap="missing-generator:unit-test:a9f9e1f4" tracker="pending-tracker" reason="Second canonical source fixture for multi-input inference and edge provenance."
pub fn handle_request(client: &dyn ExternalClient) -> Result<String, String> {
    client.fetch().map(|value| format!("handled: {value}"))
}

pub trait ExternalClient {
    fn fetch(&self) -> Result<String, String>;
}
// HANDWRITE-END
