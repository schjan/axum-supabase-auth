use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct HealthCheckResponse {
    pub version: String,
    pub name: String,
    pub description: String,
}
