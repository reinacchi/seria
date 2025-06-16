/// src/client.rs
use crate::{error::SeriaError, gateway::{GatewayClient, GatewayConfig}, http::{HttpClient, HttpConfig}};

pub struct SeriaClient {
    pub http: HttpClient,
    pub gateway: GatewayClient,
}

impl SeriaClient {
    pub fn new(http: HttpClient, gateway: GatewayClient) -> Self {
        SeriaClient { http, gateway }
    }

    pub async fn connect(&mut self) -> Result<(), SeriaError> {
        self.gateway.connect().await
    }
}

pub struct SeriaClientBuilder {
    token: Option<String>,
}

impl SeriaClientBuilder {
    pub fn new() -> Self {
        SeriaClientBuilder { token: None }
    }

    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    pub fn build(self) -> Result<SeriaClient, SeriaError> {
        let token = self.token.ok_or_else(|| {
            SeriaError::Other("Token must be provided".into())
        })?;

        let http_config = HttpConfig::new(&token)?;
        let gateway_config = GatewayConfig::new(&token)?;

        Ok(SeriaClient {
            http: HttpClient::new(http_config),
            gateway: GatewayClient::new(gateway_config),
        })
    }
}
