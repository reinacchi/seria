use crate::{
    error::{SeriaError, SeriaResult},
    gateway::{GatewayClient, GatewayConfig},
    http::{HttpClient, HttpConfig},
};

/// Represents the main Seria client.
pub struct SeriaClient {
    /// The HTTP client.
    pub http: HttpClient,
    /// The gateway client.
    pub gateway: GatewayClient,
}

impl SeriaClient {
    /// Create a new client instance.
    pub fn new(http: HttpClient, gateway: GatewayClient) -> Self {
        SeriaClient { http, gateway }
    }

    /// Connect the bot to the gateway.
    pub async fn connect(&mut self) -> SeriaResult<()> {
        self.gateway.connect().await
    }
}

/// Represents a builder pattern for constructing a SeriaClient.
pub struct SeriaClientBuilder {
    token: Option<String>,
}

impl SeriaClientBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        SeriaClientBuilder { token: None }
    }

    /// The bot token.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Build a Seria client.
    pub fn build(self) -> SeriaResult<SeriaClient> {
        let token = self
            .token
            .ok_or_else(|| SeriaError::Other("Token must be provided".into()))?;

        let http_config = HttpConfig::new(&token)?;
        let gateway_config = GatewayConfig::new(&token)?;

        Ok(SeriaClient {
            http: HttpClient::new(http_config),
            gateway: GatewayClient::new(gateway_config),
        })
    }
}
