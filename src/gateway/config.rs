use crate::error::SeriaError;

#[derive(Clone, Debug)]
pub struct GatewayConfig {
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
            token,
            ws_url: ws_url.into(),
        })
    }
}
