use async_channel::{self, Receiver, Sender};
use futures::{SinkExt, Stream, StreamExt};
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
    heartbeat_interval: Duration,
    last_heartbeat: (Instant, Instant),
    client_sender: Sender<ClientEvent>,
    client_receiver: Receiver<ClientEvent>,
    server_sender: Sender<Result<GatewayEvent, SeriaError>>,
    server_receiver: Receiver<Result<GatewayEvent, SeriaError>>,
}

impl GatewayClient {
    pub fn new(config: GatewayConfig) -> Self {
        let (client_sender, client_receiver) = async_channel::unbounded();
        let (server_sender, server_receiver) = async_channel::unbounded();

        Self {
            config,
            heartbeat_interval: Duration::from_secs(30),
            last_heartbeat: (Instant::now(), Instant::now()),
            client_receiver,
            client_sender,
            server_receiver,
            server_sender,
        }
    }

    pub async fn connect(&mut self) -> Result<(), SeriaError> {
        let (mut stream, _) = connect_async(&self.config.ws_url).await?;
        let client_receiver = self.client_receiver.clone();
        let server_sender = self.server_sender.clone();

        spawn(Self::heartbeat(
            self.client_sender.clone(),
            self.heartbeat_interval,
        ));

        spawn(async move {
            use futures::pin_mut;
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
        });

        self.send(ClientEvent::Authenticate {
            token: self.config.token.clone(),
        })
        .await
        .map_err(|_| SeriaError::Other("Failed to send authentication event".into()))
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
