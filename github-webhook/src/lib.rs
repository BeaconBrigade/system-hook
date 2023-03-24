//! # Github Webhook
//!
//! Contains types for github webhooks
//!
//! ## Features
//!
//! * axum: enable the axum feature to get extractor implementations

mod types;
#[cfg(feature = "axum")]
mod axum;

pub use types::*;
