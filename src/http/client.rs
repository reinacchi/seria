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
    models::{Message, MessageSend, MessageEdit, MessageReplyIntent},
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

    /// Sends a message in the specified channel.
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

    /// Edits a message in the specified channel.
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

    /// Replies to a certain message in the specified channel.
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
