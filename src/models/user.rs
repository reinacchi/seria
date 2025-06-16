use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::models::{Attachment, Id};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct UserStatus {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,

}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum Presence {
    Online,
    Invisible,
    Idle,
    Busy,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub enum UserFields {
    Avatar,
    StatusText,
    StatusPresence,
    ProfileContent,
    ProfileBackground,
    DisplayName,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UserUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<UserStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<UserProfileUpdate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<Id>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove: Option<UserFields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badges: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<u32>,

}

#[derive(Clone, Debug, Default, Serialize)]
pub struct UserProfileUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<Id>,
}

bitflags! {
    #[derive(Clone, Debug, PartialEq, Deserialize, Default)]
    #[serde(transparent)]
    pub struct UserBadges: u32 {
        const Developer = 1;
        const Translator = 2;
    }
}

bitflags! {
    #[derive(Clone, Debug, PartialEq, Deserialize, Default)]
    #[serde(transparent)]
    pub struct UserFlags: u32 {
        const Suspended = 1;
        const Deleted = 2;
        const Banned = 4;
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct User {
    #[serde(rename = "_id")]
    pub id: Id,
    pub username: String,
    pub avatar: Option<Attachment>,
    pub status: Option<UserStatus>,
    pub online: bool,
    #[serde(default)]
    pub badges: UserBadges,
    #[serde(default)]
    pub flags: UserFlags,
}
