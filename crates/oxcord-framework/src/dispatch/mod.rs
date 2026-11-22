//! Command dispatch logic.
//!
//! [`Dispatcher`] receives raw invocation data and routes it to the
//! matching [`Command`] handler, running pre-flight checks (cooldowns,
//! guild-only, owner-only, permissions) before calling the user function.

use std::collections::HashSet;
use std::sync::Arc;

use oxcord_model::id::UserId;
use tracing::{debug, warn};

use crate::command::Command;
use crate::context::{Context, FrameworkHttp, InvocationData};
use crate::cooldown::CooldownManager;
use crate::error::FrameworkError;
use crate::framework::FrameworkOptions;

/// Internal invocation request built by [`Framework`][crate::Framework] before
/// handing off to the dispatcher.
pub struct Invocation<'a> {
    /// The command name exactly as invoked (without prefix).
    pub command_name: &'a str,
    /// The user who invoked the command.
    pub author: &'a oxcord_model::user::User,
    /// Raw invocation data.
    pub data: &'a InvocationData,
    /// A lightweight HTTP handle.
    pub http: Arc<FrameworkHttp>,
}

/// Routes invocations to registered commands.
pub struct Dispatcher {
    commands: Vec<Command>,
    owners: HashSet<UserId>,
    cooldowns: dashmap::DashMap<String, CooldownManager>,
}

impl Dispatcher {
    /// Build a new dispatcher from framework options.
    #[must_use]
    pub fn new(options: &FrameworkOptions) -> Self {
        Self {
            commands: options.commands.clone(),
            owners: options.owners.clone(),
            cooldowns: dashmap::DashMap::new(),
        }
    }

    /// Dispatch an invocation, running all pre-flight checks.
    ///
    /// # Errors
    ///
    /// Returns the first [`FrameworkError`] encountered (cooldown, permission
    /// check, or a propagated handler error).
    pub async fn dispatch(&self, inv: Invocation<'_>) -> Result<(), FrameworkError> {
        let cmd = self
            .find_command(inv.command_name)
            .ok_or_else(|| FrameworkError::UnknownCommand(inv.command_name.to_owned()))?;

        // --- Pre-flight: guild-only ---
        let guild_id = match inv.data {
            InvocationData::Slash {
                guild_id, ..
            }
            | InvocationData::Prefix {
                guild_id, ..
            } => *guild_id,
        };
        if cmd.guild_only == Some(true) && guild_id.is_none() {
            return Err(FrameworkError::GuildOnly);
        }

        // --- Pre-flight: owner-only ---
        if cmd.owner_only && !self.owners.contains(&inv.author.id) {
            return Err(FrameworkError::OwnerOnly);
        }

        // --- Pre-flight: cooldown ---
        let channel_id = inv.http.channel_id;
        if let Some(mgr) = self.cooldowns.get(&cmd.name) {
            if let Some(remaining) = mgr.check(inv.author.id, guild_id, channel_id) {
                return Err(FrameworkError::Cooldown {
                    remaining_secs: remaining.as_secs_f64(),
                });
            }
        }

        // Record invocation before running (prevents double-trigger spam).
        if let Some(mgr) = self.cooldowns.get(&cmd.name) {
            mgr.record(inv.author.id, guild_id, channel_id);
        }

        // --- Dispatch ---
        let Some(handler) = &cmd.handler else {
            warn!(command = %cmd.name, "command has no handler registered");
            return Err(FrameworkError::Other(format!("'{}' has no handler", cmd.name)));
        };

        let ctx = Context {
            author: inv.author,
            channel_id,
            guild_id,
            invocation: inv.data,
            http: inv.http,
        };

        debug!(command = %cmd.name, "dispatching command");
        handler.call(ctx).await
    }

    /// Register a cooldown manager for a command by name.
    pub fn register_cooldown(&self, command_name: impl Into<String>, mgr: CooldownManager) {
        self.cooldowns.insert(command_name.into(), mgr);
    }

    fn find_command(&self, name: &str) -> Option<&Command> {
        self.commands.iter().find(|c| c.name == name || c.aliases.iter().any(|a| a == name))
    }
}
