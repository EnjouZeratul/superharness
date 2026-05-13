//! # Channel Adapters
//!
//! 渠道适配器实现。

mod cli;
mod http;
mod websocket;

pub use cli::CliChannel;
pub use http::HttpChannel;
pub use websocket::WebSocketChannel;
