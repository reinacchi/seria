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
use futures::StreamExt;
use seria::{
    client::SeriaClientBuilder,
    http::HttpClient,
    models::{GatewayEvent, MessageSend},
    SeriaResult,
};
use std::{env, sync::Arc};
use tracing::error;

async fn handle_event(event: GatewayEvent, http: Arc<HttpClient>) {
    match event {
        GatewayEvent::Ready => {
            if let Ok(user) = http.get_self().await {
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

                if let Err(e) = http.send_message(&message.channel, payload).await {
                    error!("Failed to send message: {}", e);
                }
            }
        }
        _ => {}
    }
}

#[tokio::main]
async fn main() -> SeriaResult<()> {
    let token = env::var("REVOLT_TOKEN")?;

    let mut client = SeriaClientBuilder::new().token(&token).build()?;

    let http = Arc::new(client.http.clone());

    client.connect().await?;

    let mut event_stream = pin!(client.gateway);

    while let Some(event) = event_stream.next().await {
        let event = event?;

        tokio::spawn(handle_event(event, Arc::clone(&http)));
    }

    Ok(())
}
```

## Useful Links

- [The official Seria Revolt server](https://rvlt.gg/g65YG8CA) - A place where you can receive support and updates.
- [The website](https://seria.2rkf.me) - The official website and documentation for Seria.

## License

Please refer to the [LICENSE](https://github.com/reinacchi/seria/blob/master/LICENSE) file.
