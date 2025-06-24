# Seria

![seria logo](logo.png)

- **⚠️ This library is heavily Work in Progress!! Please do not use yet.**

**Seria** is a Rust-based library for interacting with Revolt.

## Installation

Seria supports a MSRV of **Rust 1.76 or later**.

```toml
# Add crate to your Cargo.toml
[dependencies]
seria = "*"
tokio = { version = "*", features = ["macros", "rt-multi-thread"] }
```

## Ping Pong Example

```rs
use seria::{
    client::{SeriaClient, SeriaClientBuilder},
    models::{GatewayEvent, MessageSend},
    SeriaResult,
    StreamExt,
};
use std::{pin::pin, sync::Arc};
use tracing::{error, warn};

async fn handle_event(event: GatewayEvent, client: Arc<SeriaClient>) {
    match event {
        GatewayEvent::Ready => {
            if let Ok(user) = client.http.get_self().await {
                println!("{}#{} is Ready!", user.username, user.discriminator);
            }
        }
        GatewayEvent::Message(message) => {
            let content = message.content.to_string();

            if content == "!ping" {
                let payload = MessageSend {
                    content: "Pong!".to_string(),
                    ..Default::default()
                };

                if let Err(e) = client.http.send_message(&message.channel, payload).await {
                    error!("Failed to send message: {}", e);
                }
            }
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> SeriaResult<()> {
    tracing_subscriber::fmt::init();

    let token = "REVOLT_TOKEN";

    let mut client = SeriaClientBuilder::new().
        token(token)
        .build()?;

    client.connect().await?;

    let client = Arc::new(client);

    let mut event_stream = pin!(client.gateway.clone());

    while let Some(item) = event_stream.next().await {
        match item {
            Ok(event) => {
                let client = Arc::clone(&client);
                tokio::spawn(async move {
                    handle_event(event, client).await;
                });
            }
            Err(e) => {
                warn!(error = ?e, "Failed to receive event");
            }
        }
    }

    Ok(())
}
```

## Useful Links

- [The official Seria Revolt server](https://rvlt.gg/g65YG8CA) - A place where you can receive support and updates.
- [The website](https://seria.2rkf.me) - The official website and documentation for Seria.

## License

Please refer to the [LICENSE](https://github.com/reinacchi/seria/blob/master/LICENSE) file.
