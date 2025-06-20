# Seria

- **⚠️ This library is heavily Work in Progress!! Please do not use.**

**Seria** is a lightweight Revolt client library built for the Rust ecosystem.

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
    error::SeriaError,
    http::HttpClient,
    models::{GatewayEvent, MessageSend},
};
use std::{env, sync::Arc};
use tracing::error;

async fn handle_event(event: GatewayEvent, http: Arc<HttpClient>) {
    match event {
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
async fn main() -> Result<(), SeriaError> {
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

- [The official Seria Revolt server]() - A place where you can receive support and updates.

## License

Please refer to the [LICENSE](https://github.com/reinacchi/seria/blob/dev/LICENSE) file.
