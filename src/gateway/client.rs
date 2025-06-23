use async_channel::{self, Receiver, Sender};
use futures::{pin_mut, SinkExt, Stream, StreamExt};
use std::time::{Duration, Instant};
use tokio::{select, spawn, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    error::{SeriaError, SeriaResult},
    gateway::GatewayConfig,
    models::{ClientEvent, GatewayEvent},
};

#[derive(Debug, Clone)]
pub struct GatewayClient {
    config: GatewayConfig,
    last_heartbeat: (Instant, Instant),
    client_sender: Sender<ClientEvent>,
    client_receiver: Receiver<ClientEvent>,
    server_sender: Sender<Result<GatewayEvent, SeriaError>>,
    server_receiver: Receiver<Result<GatewayEvent, SeriaError>>,
    is_connected: bool,
}

impl GatewayClient {
    pub fn new(config: GatewayConfig) -> Self {
        let (client_sender, client_receiver) = async_channel::unbounded();
        let (server_sender, server_receiver) = async_channel::unbounded();

        Self {
            config,
            last_heartbeat: (Instant::now(), Instant::now()),
            client_receiver,
            client_sender,
            server_receiver,
            server_sender,
            is_connected: false,
        }
    }

    pub async fn connect(&mut self) -> SeriaResult<()> {
        if self.is_connected {
            return Ok(());
        }

        self.is_connected = true;

        loop {
            match self.try_connect().await {
                Ok(_) => {
                    self.config.reconnect_attempts = 0; // Reset attempts on successful connection
                    return Ok(());
                }
                Err(e) => {
                    if self.config.reconnect_attempts >= self.config.max_reconnect_attempts {
                        return Err(e);
                    }

                    self.config.reconnect_attempts += 1;
                    let delay = self.config.reconnect_delay * self.config.reconnect_attempts as u32;
                    sleep(delay).await;
                }
            }
        }
    }

    async fn try_connect(&mut self) -> SeriaResult<()> {
        let (stream, _) = match connect_async(&self.config.ws_url).await {
            Ok((stream, response)) => (stream, response),
            Err(e) => {
                return Err(e.into());
            }
        };

        let client_receiver = self.client_receiver.clone();
        let server_sender = self.server_sender.clone();
        let heartbeat_sender = self.client_sender.clone();

        self.send(ClientEvent::Authenticate {
            token: self.config.token.clone(),
        })
        .await
        .map_err(|_e| SeriaError::Other("Failed to send authentication event".into()))?;

        let heartbeat_task = spawn({
            let interval = self.config.heartbeat_interval;
            async move {
                let _ = Self::heartbeat(heartbeat_sender, interval).await;
            }
        });

        let (mut write_stream, mut read_stream) = stream.split();

        let write_task = spawn({
            let server_sender = server_sender.clone();
            async move {
                pin_mut!(client_receiver);

                while let Some(event) = client_receiver.next().await {
                    let msg = match serde_json::to_string(&event) {
                        Ok(json) => Message::Text(json.into()),
                        Err(e) => {
                            let _ = server_sender
                                .send(Err(SeriaError::Other(format!(
                                    "Serialization error: {}",
                                    e
                                ))))
                                .await;
                            continue;
                        }
                    };

                    if let Err(e) = write_stream.send(msg).await {
                        let _ = server_sender.send(Err(e.into())).await;
                        break;
                    }
                }
            }
        });

        let read_task = spawn({
            let server_sender = server_sender.clone();
            async move {
                while let Some(msg) = read_stream.next().await {
                    let event = match msg {
                        Ok(msg) => match msg {
                            Message::Text(text) => {
                                match serde_json::from_str::<GatewayEvent>(&text) {
                                    Ok(GatewayEvent::Pong) => continue,
                                    Ok(event) => Ok(event),
                                    Err(e) => Err(SeriaError::Other(format!(
                                        "Deserialization error: {}",
                                        e
                                    ))),
                                }
                            }
                            Message::Close(_) => {
                                break;
                            }
                            _ => continue,
                        },
                        Err(e) => Err(e.into()),
                    };

                    if server_sender.send(event).await.is_err() {
                        break;
                    }
                }

                let _ = server_sender
                    .send(Err(SeriaError::Other("WebSocket disconnected".to_string())))
                    .await;
            }
        });

        spawn(async move {
            select! {
                _ = heartbeat_task => {},
                _ = write_task => {},
                _ = read_task => {},
            }

            // Attempt to reconnect when any task ends
            let _ = server_sender
                .send(Err(SeriaError::Other(
                    "WebSocket connection lost, attempting to reconnect".to_string(),
                )))
                .await;
        });

        Ok(())
    }

    pub async fn send(&self, event: ClientEvent) -> SeriaResult<()> {
        self.client_sender
            .send(event)
            .await
            .map_err(|e| SeriaError::Other(format!("Failed to send event to client: {}", e)))
    }

    pub fn latency(&self) -> Duration {
        let (last_ping, last_pong) = self.last_heartbeat;
        if last_ping > last_pong {
            last_ping - last_pong
        } else {
            last_pong - last_ping
        }
    }

    async fn heartbeat(sender: Sender<ClientEvent>, interval: Duration) -> Result<(), SeriaError> {
        loop {
            if let Err(_e) = sender.send(ClientEvent::Ping { data: 0 }).await {
                break;
            }
            sleep(interval).await;
        }
        Ok(())
    }
}

impl Stream for GatewayClient {
    type Item = SeriaResult<GatewayEvent>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let pinned_receiver = unsafe { self.map_unchecked_mut(|s| &mut s.server_receiver) };
        pinned_receiver.poll_next(cx)
    }
}
