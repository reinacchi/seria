use async_channel::{self, Receiver, Sender};
use futures::{pin_mut, SinkExt, Stream, StreamExt};
use std::time::{Duration, Instant};
use tokio::{select, spawn, time::sleep};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message},
};

use crate::{
    error::{SeriaError, SeriaResult},
    gateway::GatewayConfig,
    models::{ClientEvent, GatewayEvent},
};

#[derive(Debug, Clone)]
pub struct GatewayClient {
    config: GatewayConfig,
    pub last_heartbeat: (Instant, Instant),
    client_sender: Sender<ClientEvent>,
    client_receiver: Receiver<ClientEvent>,
    server_sender: Sender<Result<GatewayEvent, SeriaError>>,
    server_receiver: Receiver<Result<GatewayEvent, SeriaError>>,
    pub is_connected: bool,
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

        let mut client = self.clone();
        spawn(async move {
            loop {
                match client.try_connect().await {
                    Ok(_) => {
                        client.config.reconnect_attempts = 0;
                    }
                    Err(e) => {
                        let _ = client
                            .server_sender
                            .send(Err(SeriaError::Other(format!(
                                "Connection failed. {}, retrying in {}s",
                                e,
                                client.config.reconnect_delay.as_secs()
                                    * client.config.reconnect_attempts as u64
                            ))))
                            .await;
                        client.is_connected = false;
                        client.config.reconnect_attempts += 1;

                        let delay = std::cmp::min(
                            client.config.reconnect_delay * client.config.reconnect_attempts as u32,
                            Duration::from_secs(60),
                        );
                        sleep(delay).await;
                    }
                }
            }
        });

        self.is_connected = true;
        Ok(())
    }

    async fn try_connect(&mut self) -> SeriaResult<()> {
        let (stream, _) = match connect_async(&self.config.ws_url).await {
            Ok((stream, response)) => (stream, response),
            Err(e) => {
                return Err(handle_websocket_error(e));
            }
        };

        self.is_connected = true;
        self.config.reconnect_attempts = 0;

        self.send(ClientEvent::Authenticate {
            token: self.config.token.clone(),
        })
        .await
        .map_err(|_e| SeriaError::Other("Failed to send authentication event".into()))?;

        let client_receiver = self.client_receiver.clone();
        let server_sender = self.server_sender.clone();
        let heartbeat_sender = self.client_sender.clone();

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
                        let _ = server_sender
                            .send(Err(handle_websocket_error(e).into()))
                            .await;
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
                        Err(e) => Err(handle_websocket_error(e).into()),
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

        select! {
            _ = heartbeat_task => Err(SeriaError::Other("Heartbeat task terminated".into())),
            _ = write_task => Err(SeriaError::Other("Write task terminated".into())),
            _ = read_task => Err(SeriaError::Other("Read task terminated".into())),
        }
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
            last_pong - last_pong
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

fn handle_websocket_error(err: WsError) -> SeriaError {
    match &err {
        WsError::AlreadyClosed => SeriaError::Other("WebSocket already closed".to_string()),
        WsError::Io(io_err) if io_err.raw_os_error() == Some(104) => {
            SeriaError::Other("Connection reset by peer".to_string())
        }
        WsError::Io(io_err) if io_err.raw_os_error() == Some(10054) => {
            SeriaError::Other("Connection forcibly closed by remote host".to_string())
        }
        _ => SeriaError::WebSocket(err),
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
