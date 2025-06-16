use {
    reqwest::{
        header::{HeaderMap, HeaderValue},
        Client,
    },
    serde::{de::DeserializeOwned, ser::Serialize},
};

use crate::{
    error::{SeriaError as Error, SeriaResult as Result},
    http::{endpoint::Endpoint, HttpConfig},
    models::{Message, MessageCreate},
};

#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    config: HttpConfig,
}

impl HttpClient {
    pub fn new(config: HttpConfig) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert("x-bot-token", HeaderValue::from_str(&config.token).unwrap());

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

    pub async fn get<T: DeserializeOwned>(&self, path: impl AsRef<str>) -> Result<T> {
        let response = self.client.get(self.make_url(path)).send().await?;

        if !response.status().is_success() {
            return Err(Error::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    pub async fn post<T: DeserializeOwned, U: Serialize>(
        &self,
        path: impl AsRef<str>,
        payload: U,
    ) -> Result<T> {
        let response = self
            .client
            .post(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::FailedRequest(response));
        }

        let payload = response.json().await?;

        Ok(payload)
    }

    pub async fn put<T: Serialize>(&self, path: impl AsRef<str>, payload: T) -> Result {
        let response = self
            .client
            .put(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::FailedRequest(response));
        }

        Ok(())
    }

    pub async fn patch<T: Serialize>(&self, path: impl AsRef<str>, payload: T) -> Result {
        let response = self
            .client
            .patch(self.make_url(path))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::FailedRequest(response));
        }

        Ok(())
    }

    pub async fn delete(&self, path: impl AsRef<str>) -> Result {
        let response = self.client.delete(self.make_url(path)).send().await?;

        if !response.status().is_success() {
            return Err(Error::FailedRequest(response));
        }

        Ok(())
    }

    /// Creates a message in the specified channel.
    pub async fn create_message(
        &self,
        channel_id: &str,
        payload: impl Into<MessageCreate>,
    ) -> Result<Message> {
        self.post(Endpoint::ChannelMessageSend(channel_id.to_string()).path(), payload.into())
            .await
    }
}
