use std::fmt;

/// Base URL for the Revolt API
const BASE_URL: &str = "https://api.revolt.chat";

#[derive(Debug, Clone)]
pub enum Endpoint {
    ChannelMessageSend(String),
    ChannelMessageEdit(String, String),
}

impl Endpoint {
    /// Returns the HTTP method typically used for this endpoint
    pub fn method(&self) -> &'static str {
        match self {
            Endpoint::ChannelMessageSend(_) => "POST",
            Endpoint::ChannelMessageEdit(_, _) => "PATCH",
        }
    }

    /// Returns the path component of the endpoint URL
    pub fn path(&self) -> String {
        match self {
            Endpoint::ChannelMessageSend(channel_id) => {
                format!("/channels/{}/messages", channel_id)
            }
            Endpoint::ChannelMessageEdit(channel_id, message_id) => {
                format!("/channels/{}/messages/{}", channel_id, message_id)
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
