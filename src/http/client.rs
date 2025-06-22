use {
    reqwest::{
        header::{HeaderMap, HeaderValue},
        Client,
    },
    serde::{de::DeserializeOwned, ser::Serialize},
};

use crate::{
    error::{SeriaError, SeriaResult},
    http::{endpoint::Endpoint, HttpConfig},
    models::{
        FlagResponse, Message, MessageEdit, MessageReplyIntent, MessageSend, User, UserUpdate,
    },
};

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: HttpConfig,
}

impl HttpClient {
    pub fn new(config: HttpConfig) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("X-Bot-Token", HeaderValue::from_str(&config.token).unwrap());

        let client = Client::builder().default_headers(headers).build().unwrap();

        Self { client, config }
    }

    fn make_url(&self, path: impl AsRef<str>) -> String {
        format!(
            "{}/{}",
            self.config.api_url.trim_end_matches('/'),
            path.as_ref()
        )
    }

    pub async fn get<T: DeserializeOwned>(&self, path: impl AsRef<str>) -> SeriaResult<T> {
        let response = self.client.get(self.make_url(path)).send().await?;

        if !response.status().is_success() {
            return Err(SeriaError::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    pub async fn post<T: DeserializeOwned, U: Serialize>(
        &self,
        path: impl AsRef<str>,
        payload: U,
    ) -> SeriaResult<T> {
        let response = self
            .client
            .post(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SeriaError::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    pub async fn put<T: Serialize>(&self, path: impl AsRef<str>, payload: T) -> SeriaResult {
        let response = self
            .client
            .put(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SeriaError::FailedRequest(response));
        }

        Ok(())
    }

    pub async fn patch<T: Serialize, R: DeserializeOwned>(
        &self,
        path: impl AsRef<str>,
        payload: T,
    ) -> SeriaResult<R> {
        let response = self
            .client
            .patch(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(SeriaError::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    pub async fn delete(&self, path: impl AsRef<str>) -> SeriaResult {
        let response = self.client.delete(self.make_url(path)).send().await?;

        if !response.status().is_success() {
            return Err(SeriaError::FailedRequest(response));
        }

        Ok(())
    }

    // User-related methods
    /// Get properties of the bot user.
    pub async fn get_self(&self) -> SeriaResult<User> {
        self.get(Endpoint::User("@me".to_string()).path()).await
    }

    /// Edit a user.
    pub async fn edit_user(
        &self,
        user_id: &str,
        payload: impl Into<UserUpdate>,
    ) -> SeriaResult<User> {
        self.patch(Endpoint::User(user_id.to_string()).path(), payload.into())
            .await
    }

    /// Get properties of the targeted user.
    pub async fn get_user(&self, user_id: &str) -> SeriaResult<User> {
        self.get(Endpoint::User(user_id.to_string()).path()).await
    }

    /// Get the flags of the targeted user.
    pub async fn get_user_flags(&self, user_id: &str) -> SeriaResult<FlagResponse> {
        self.get(Endpoint::UserFlags(user_id.to_string()).path())
            .await
    }

    // Message-related methods
    /// Send a message in the specified channel.
    pub async fn send_message(
        &self,
        channel_id: &str,
        payload: impl Into<MessageSend>,
    ) -> SeriaResult<Message> {
        self.post(
            Endpoint::ChannelMessageSend(channel_id.to_string()).path(),
            payload.into(),
        )
        .await
    }

    /// Edit a message in the specified channel.
    pub async fn edit_message(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: impl Into<MessageEdit>,
    ) -> SeriaResult<Message> {
        self.patch(
            Endpoint::ChannelMessageEdit(channel_id.to_string(), message_id.to_string()).path(),
            payload.into(),
        )
        .await
    }

    /// Reply to a certain message in the specified channel.
    pub async fn reply_message(
        &self,
        channel_id: &str,
        message_id: &str,
        payload: impl Into<MessageSend>,
        mention: bool,
    ) -> SeriaResult<Message> {
        let reply_intent = MessageReplyIntent {
            id: message_id.to_string().into(),
            mention,
            fail_if_not_exists: true,
        };

        let mut message_payload: MessageSend = payload.into();
        message_payload.replies.push(reply_intent);

        self.send_message(channel_id, message_payload).await
    }
}
