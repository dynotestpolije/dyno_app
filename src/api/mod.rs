mod connection;
mod handler;
mod session;

pub use connection::ApiConnection;
use dyno_types::once_cell::sync::Lazy;

pub(self) const URL_SERVER: Lazy<String> =
    Lazy::new(|| std::env::var("DYNO_SERVER_URL").unwrap_or("localhost:1406".to_owned()));
