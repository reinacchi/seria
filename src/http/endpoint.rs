use std::fmt;

/// Base URL for the Revolt API
const BASE_URL: &str = "https://api.revolt.chat";

/// Enum representing API endpoints (simplified to just ChannelMessageSend for now)
#[derive(Debug, Clone)]
pub enum Endpoint {
    ChannelMessageSend(String),
}

impl Endpoint {
    /// Returns the HTTP method typically used for this endpoint
    pub fn method(&self) -> &'static str {
        match self {
            Endpoint::ChannelMessageSend(_) => "POST",
        }
    }

    /// Returns the path component of the endpoint URL
    pub fn path(&self) -> String {
        match self {
            Endpoint::ChannelMessageSend(channel_id) => {
                format!("/channels/{}/messages", channel_id)
            }
        }
    }

    /// Returns the full URL for the endpoint
    pub fn url(&self) -> String {
        format!("{}{}", BASE_URL, self.path())
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.method(), self.url())
    }
}
