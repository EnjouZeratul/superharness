//! # SuperHarness Layer 1: Foundation
//!
//! 基础设施层，为上层提供核心能力。

pub mod cache_manager;
pub mod config_manager;
pub mod cost_tracker;
pub mod embeddings;
pub mod error_handler;
pub mod event_bus;
pub mod llm_client;
pub mod observability;
pub mod storage_engine;
pub mod streaming;

pub use cache_manager::CacheManager;
pub use config_manager::{ConfigManager, GlobalSettings, ProviderConfig};
pub use cost_tracker::CostTracker;
pub use error_handler::{ErrorHandler, ShError, ShResult};
pub use event_bus::EventBus;
pub use llm_client::LlmClient;
pub use observability::Observability;
pub use storage_engine::StorageEngine;
pub use streaming::StreamHandler;
