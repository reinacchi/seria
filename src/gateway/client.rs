use async_channel::{self, Receiver, Sender};
use futures::{pin_mut, SinkExt, Stream, StreamExt};
use std::time::{Duration, Instant};
use tokio::{select, spawn, time::sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::{
    error::SeriaError,
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

    pub async fn connect(&mut self) -> Result<(), SeriaError> {
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

    async fn try_connect(&mut self) -> Result<(), SeriaError> {
        let (mut stream, _) = connect_async(&self.config.ws_url).await?;
        let client_receiver = self.client_receiver.clone();
        let server_sender = self.server_sender.clone();
        let heartbeat_sender = self.client_sender.clone();

        let heartbeat_task = spawn(Self::heartbeat(
            heartbeat_sender,
            self.config.heartbeat_interval,
        ));

        let connection_task = spawn(async move {
            pin_mut!(client_receiver);

            loop {
                select! {
                    Some(event) = client_receiver.next() => {
                        let msg = match serde_json::to_string(&event) {
                            Ok(json) => Message::Text(json.into()),
                            Err(e) => {
                                let _ = server_sender.send(Err(SeriaError::Other(format!("Serialization error: {}", e)))).await;
                                continue;
                            }
                        };
                        if let Err(e) = stream.send(msg).await {
                            let _ = server_sender.send(Err(e.into())).await;
                            break;
                        }
                    },
                    Some(msg) = stream.next() => {
                        let event = match msg {
                            Ok(msg) => match msg.to_text() {
                                Ok(text) => {
                                    let value: serde_json::Value = match serde_json::from_str(text) {
                                        Ok(v) => v,
                                        Err(e) => {
                                            let _ = server_sender.send(Err(SeriaError::Other(format!("Deserialization error: {}", e)))).await;
                                            continue;
                                        }
                                    };

                                    // Handle heartbeat acknowledgment
                                    if let Ok(GatewayEvent::Pong) = serde_json::from_value(value.clone()) {
                                        continue;
                                    }

                                    match serde_json::from_value(value.clone()) {
                                        Ok(event) => Ok(event),
                                        Err(_) => continue,
                                    }
                                },
                                Err(e) => Err(e.into()),
                            },
                            Err(e) => Err(e.into()),
                        };

                        if let Ok(_inner_event) = &event {
                            if server_sender.send(event).await.is_err() {
                                break;
                            }
                        }
                    },
                    else => break,
                }
            }

            let _ = server_sender
                .send(Err(SeriaError::Other(
                    "WebSocket disconnected.".to_string(),
                )))
                .await;
        });

        self.send(ClientEvent::Authenticate {
            token: self.config.token.clone(),
        })
        .await
        .map_err(|_| SeriaError::Other("Failed to send authentication event".into()))?;

        spawn(async move {
            select! {
                _ = heartbeat_task => {},
                _ = connection_task => {},
            }
        });

        Ok(())
    }

    pub async fn send(&self, event: ClientEvent) -> Result<(), SeriaError> {
        self.client_sender
            .send(event)
            .await
            .map_err(|_| SeriaError::Other("Failed to send event to client".into()))
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
            let _ = sender.send(ClientEvent::Ping { data: 0 }).await;
            sleep(interval).await;
        }
    }
}

impl Stream for GatewayClient {
    type Item = Result<GatewayEvent, SeriaError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let pinned_receiver = unsafe { self.map_unchecked_mut(|s| &mut s.server_receiver) };
        pinned_receiver.poll_next(cx)
    }
}
