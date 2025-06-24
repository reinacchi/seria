#[derive(Debug, Clone)]
pub enum Endpoint {
    BotInvite(String),
    ChannelMessageSend(String),
    ChannelMessageEdit(String, String),
    User(String),
    UserFlags(String),
    UserUsername(String),
    UserDefaultAvatar(String),
    UserProfile(String),
}

impl Endpoint {
    /// Returns the path component of the endpoint URL
    pub fn path(&self) -> String {
        match self {
            Endpoint::BotInvite(bot_id) => {
                format!("/bots/{}/invite", bot_id)
            }
            Endpoint::ChannelMessageSend(channel_id) => {
                format!("/channels/{}/messages", channel_id)
            }
            Endpoint::ChannelMessageEdit(channel_id, message_id) => {
                format!("/channels/{}/messages/{}", channel_id, message_id)
            }
            Endpoint::User(user_id) => {
                format!("/users/{}", user_id)
            }
            Endpoint::UserFlags(user_id) => {
                format!("/users/{}/flags", user_id)
            }
            Endpoint::UserUsername(user_id) => {
                format!("/users/{}/username", user_id)
            }
            Endpoint::UserDefaultAvatar(user_id) => {
                format!("/users/{}/default_avatar", user_id)
            }
            Endpoint::UserProfile(user_id) => {
                format!("/users/{}/profile", user_id)
            }
        }
    }
}
