//! `oxcord-framework` — A high-level, poise-style command framework for Oxcord.
//!
//! # Overview
//!
//! This crate provides a unified command framework that integrates both **slash (application)**
//! and **prefix (text)** commands through a single function annotation, similar to [poise] for
//! serenity.
//!
//! # Quick Start
//!
//! ```rust,no_run,ignore
//! use oxcord_framework::{Framework, FrameworkOptions, Context, Command};
//!
//! #[oxcord_macros::command(slash_command, prefix_command)]
//! async fn ping(ctx: Context<'_>) -> oxcord_framework::Result<()> {
//!     ctx.say("Pong!").await?;
//!     Ok(())
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let framework = Framework::builder()
//!         .options(FrameworkOptions {
//!             commands: vec![ping()],
//!             prefix: Some("!".into()),
//!             ..Default::default()
//!         })
//!         .build();
//! }
//! ```
//!
//! [poise]: https://docs.rs/poise

#![warn(
    missing_docs,
    rust_2018_idioms,
    clippy::unwrap_used,
    clippy::pedantic,
    clippy::clone_on_ref_ptr
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::doc_markdown,
    clippy::type_complexity
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod command;
pub mod context;
pub mod cooldown;
pub mod dispatch;
pub mod error;
pub mod framework;

pub use command::{
    AutocompleteFn,
    AutocompleteHandler,
    Command,
    CommandCategory,
    CommandFn,
    CommandHandler,
    CommandKind,
};
pub use context::Context;
pub use cooldown::{CooldownConfig, CooldownManager, CooldownScope};
pub use error::FrameworkError;
pub use framework::{Framework, FrameworkBuilder, FrameworkOptions};

/// Convenience result type for command handlers.
pub type Result<T, E = FrameworkError> = std::result::Result<T, E>;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
pub use oxcord_macros as macros;
