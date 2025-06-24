#[doc(hidden)]
pub use error::SeriaResult;
pub use futures::StreamExt;
pub mod error;
pub mod client;
pub mod gateway;
pub mod http;

#[cfg(feature = "cache")]
pub mod models;

#[cfg(feature = "type-store")]
pub mod type_store;
