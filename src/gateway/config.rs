use std::time::Duration;

use crate::error::SeriaError;

#[derive(Clone, Debug)]
pub struct GatewayConfig {
    pub heartbeat_interval: Duration,
    pub max_reconnect_attempts: usize,
    pub reconnect_attempts: usize,
    pub reconnect_delay: Duration,
    pub token: String,
    pub ws_url: String,
}

impl GatewayConfig {
    pub fn new(token: impl Into<String>) -> Result<Self, SeriaError> {
        let token = token.into();

        if token.is_empty() {
            return Err(SeriaError::Other("Token cannot be empty".into()));
        }

        let ws_url = if cfg!(feature = "msgpack") {
            "wss://ws.revolt.chat?format=msgpack"
        } else {
            "wss://ws.revolt.chat"
        };

        Ok(GatewayConfig {
            heartbeat_interval: Duration::from_secs(15),
            max_reconnect_attempts: 5,
            reconnect_attempts: 0,
            reconnect_delay: Duration::from_secs(5),
            token,
            ws_url: ws_url.into(),
        })
    }
}
