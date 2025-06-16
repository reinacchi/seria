use serde::{Deserialize, Serialize};

use crate::models::{message::Message, server::Server, Id};

#[derive(Debug, Serialize, Clone, PartialEq)]
#[serde(tag = "type")]
pub enum ClientEvent {
    Authenticate {
        token: String,
    },
    BeginTyping {
        channel: Id,
    },
    EndTyping {
        channel: Id,
    },
    Ping {
        data: usize,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum GatewayEvent {
    Authenticated,
    Pong,
    Ready,
    Message(Message),
    ServerCreate(Server),
    ChannelStartTyping,
    ChannelStopTyping,
    #[serde(other)]
    Unknown,
}
