use crate::error::SeriaError;

#[derive(Clone, Debug)]
pub struct HttpConfig {
    pub token: String,
    pub api_url: String,
}

impl HttpConfig {
    pub fn new(token: impl Into<String>) -> Result<Self, SeriaError> {
        let token = token.into();
        if token.is_empty() {
            return Err(SeriaError::Other("Token cannot be empty".into()));
        }
        Ok(HttpConfig {
            token,
            api_url: "https://api.revolt.chat".into(),
        })
    }
}
