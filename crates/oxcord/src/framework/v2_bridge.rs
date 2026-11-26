//! Bridges [`oxcord_framework::Framework`] (the poise-style command
//! framework) into the old-style [`Framework`] trait that
//! [`ClientBuilder::framework`] expects, so a poise-style framework can
//! actually be passed to it and receive gateway events.
//!
//! [`ClientBuilder::framework`]: crate::client::ClientBuilder::framework

use async_trait::async_trait;
use oxcord_builders::{Builder, CreateAutocompleteResponse, CreateInteractionResponse};
use oxcord_model::application::Interaction;

use super::Framework;
use crate::client::{Context, FullEvent};

/// Compute a user's resolved permissions in a channel from the gateway
/// cache, the same way Discord resolves them for interactions — combining
/// the guild's `@everyone`/role permissions with the channel's permission
/// overwrites (see [`oxcord_model::guild::Guild::user_permissions_in_`]).
///
/// Used to back [`oxcord_framework::Command::required_permissions`]/
/// `required_bot_permissions` for **prefix** commands, since plain messages
/// (unlike slash interactions) don't carry resolved permissions from
/// Discord directly. Returns `None` for DMs, or when the guild, channel or
/// member isn't in the cache yet (e.g. right after startup, before the
/// relevant gateway events have arrived) — callers should treat that as
/// "couldn't verify" rather than "has no permissions".
#[cfg(feature = "cache")]
fn compute_cached_permissions(
    cache: &crate::cache::Cache,
    guild_id: Option<oxcord_model::id::GuildId>,
    channel_id: oxcord_model::id::ChannelId,
    user_id: oxcord_model::id::UserId,
) -> Option<oxcord_model::Permissions> {
    let guild = cache.guild(guild_id?)?;
    let channel = guild.channels.get(&channel_id);
    let member = guild.members.get(&user_id)?;
    Some(oxcord_model::guild::Guild::user_permissions_in_(
        channel,
        user_id,
        &member.roles,
        guild.id,
        &guild.roles,
        guild.owner_id,
    ))
}

#[cfg(feature = "cache")]
fn compute_channel_nsfw_and_guild_cached(
    cache: &crate::cache::Cache,
    guild_id: Option<oxcord_model::id::GuildId>,
    channel_id: oxcord_model::id::ChannelId,
) -> (Option<bool>, Option<bool>) {
    let Some(gid) = guild_id else {
        // DM bukan channel NSFW guild.
        return (Some(false), None);
    };
    let Some(guild) = cache.guild(gid) else {
        return (None, Some(false));
    };
    let is_nsfw = guild.channels.get(&channel_id).map(|c| c.nsfw);
    (is_nsfw, Some(true))
}

#[async_trait]
impl Framework for oxcord_framework::Framework {
    async fn dispatch(&self, ctx: Context, event: FullEvent) {
        match event {
            FullEvent::Ready {
                ..
            } => {
                if self.should_initialize_owners() {
                    if let Err(error) = self.initialize_owners(&ctx.http).await {
                        tracing::warn!(error = %error, "oxcord-framework: gagal initialize owners");
                    }
                }
                if !self.auto_register_slash() {
                    return;
                }
                match self.register_global_commands(&ctx.http).await {
                    Ok(()) => {
                        tracing::info!(
                            commands = self.build_global_commands().len(),
                            "oxcord-framework: registered global slash commands"
                        );
                    },
                    Err(error) => {
                        tracing::warn!(error = %error, "oxcord-framework: failed to register global slash commands");
                    },
                }
            },
            FullEvent::Message {
                new_message,
            } => {
                #[cfg(feature = "cache")]
                let (bot_user_id, member_permissions, app_permissions, is_nsfw, guild_in_cache) = {
                    let bot_id = ctx.cache.current_user().id;
                    let (nsfw, in_cache) = compute_channel_nsfw_and_guild_cached(
                        &ctx.cache,
                        new_message.guild_id,
                        new_message.channel_id,
                    );
                    (
                        Some(bot_id),
                        compute_cached_permissions(
                            &ctx.cache,
                            new_message.guild_id,
                            new_message.channel_id,
                            new_message.author.id,
                        ),
                        compute_cached_permissions(
                            &ctx.cache,
                            new_message.guild_id,
                            new_message.channel_id,
                            bot_id,
                        ),
                        nsfw,
                        in_cache,
                    )
                };
                #[cfg(not(feature = "cache"))]
                let (bot_user_id, member_permissions, app_permissions, is_nsfw, guild_in_cache) =
                    (None, None, None, None, None);

                // Escape hatch: context gateway asli dioper ke command lewat
                // `oxcord_framework::Context::gateway_context::<crate::client::Context>()`.
                let gateway: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>> =
                    Some(std::sync::Arc::new(ctx.clone()));
                let Some(result) = self
                    .dispatch_full_message(
                        &new_message,
                        std::sync::Arc::clone(&ctx.http),
                        bot_user_id,
                        member_permissions,
                        app_permissions,
                        is_nsfw,
                        guild_in_cache,
                        None,
                        gateway,
                    )
                    .await
                else {
                    return;
                };
                if let Err(error) = result {
                    tracing::warn!(error = %error, "oxcord-framework: prefix command dispatch failed");
                }
            },
            FullEvent::MessageUpdate {
                new: Some(new_message), ..
            } => {
                let prefix_opts = self.prefix_options();
                let Some(tracker) = &prefix_opts.edit_tracker else {
                    return;
                };
                let update_entry = tracker.process_message_update(new_message.id);
                let reuse_message_id = if let Some(entry) = &update_entry {
                    if prefix_opts.ignore_edits_if_not_yet_responded
                        && entry.bot_response_id.is_none()
                    {
                        return;
                    }
                    let cmd = self.commands().iter().find(|c| c.name == entry.command_name);
                    if !cmd.is_some_and(|c| c.invoke_on_edit) {
                        return;
                    }
                    if cmd.is_some_and(|c| c.reuse_response) { entry.bot_response_id } else { None }
                } else {
                    if !prefix_opts.execute_untracked_edits {
                        return;
                    }
                    if let Some(window) = prefix_opts.tracking_initiation_window {
                        let age_ms = oxcord_model::Timestamp::now()
                            .timestamp_millis()
                            .saturating_sub(new_message.id.created_at().timestamp_millis());
                        if age_ms < 0 || (age_ms as u64) > window.as_millis() as u64 {
                            return;
                        }
                    }
                    None
                };

                #[cfg(feature = "cache")]
                let (bot_user_id, member_permissions, app_permissions, is_nsfw, guild_in_cache) = {
                    let bot_id = ctx.cache.current_user().id;
                    let (nsfw, in_cache) = compute_channel_nsfw_and_guild_cached(
                        &ctx.cache,
                        new_message.guild_id,
                        new_message.channel_id,
                    );
                    (
                        Some(bot_id),
                        compute_cached_permissions(
                            &ctx.cache,
                            new_message.guild_id,
                            new_message.channel_id,
                            new_message.author.id,
                        ),
                        compute_cached_permissions(
                            &ctx.cache,
                            new_message.guild_id,
                            new_message.channel_id,
                            bot_id,
                        ),
                        nsfw,
                        in_cache,
                    )
                };
                #[cfg(not(feature = "cache"))]
                let (bot_user_id, member_permissions, app_permissions, is_nsfw, guild_in_cache) =
                    (None, None, None, None, None);

                let gateway: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>> =
                    Some(std::sync::Arc::new(ctx.clone()));
                if let Some(Err(error)) = self
                    .dispatch_full_message(
                        &new_message,
                        std::sync::Arc::clone(&ctx.http),
                        bot_user_id,
                        member_permissions,
                        app_permissions,
                        is_nsfw,
                        guild_in_cache,
                        reuse_message_id,
                        gateway,
                    )
                    .await
                {
                    tracing::warn!(error = %error, "oxcord-framework: edited prefix command dispatch failed");
                }
            },
            FullEvent::MessageDelete {
                channel_id,
                deleted_message_id,
                ..
            } => {
                let Some(tracker) = &self.prefix_options().edit_tracker else {
                    return;
                };
                if let Some(entry) = tracker.process_message_delete(deleted_message_id) {
                    let should_delete = self
                        .commands()
                        .iter()
                        .find(|c| c.name == entry.command_name)
                        .is_some_and(|c| c.track_deletion);
                    if should_delete {
                        if let Some(bot_msg_id) = entry.bot_response_id {
                            if let Err(error) =
                                ctx.http.delete_message(channel_id, bot_msg_id, None).await
                            {
                                tracing::warn!(error = %error, "oxcord-framework: gagal hapus balasan saat pesan asal dihapus");
                            }
                        }
                    }
                }
            },
            FullEvent::InteractionCreate {
                interaction: Interaction::Command(command),
            } => {
                #[cfg(feature = "cache")]
                let (is_nsfw, guild_in_cache) = compute_channel_nsfw_and_guild_cached(
                    &ctx.cache,
                    command.guild_id,
                    command.channel_id,
                );
                #[cfg(not(feature = "cache"))]
                let (is_nsfw, guild_in_cache) = (None, None);

                let gateway: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>> =
                    Some(std::sync::Arc::new(ctx.clone()));
                // Kalau `on_error` dikonfigurasi, error sudah ditangani hook dan ini balik `Ok`;
                // `warn!` di bawah cuma jalan sebagai fallback kalau hook nggak dipasang.
                if let Err(error) = self
                    .dispatch_slash_with_context(
                        &command,
                        std::sync::Arc::clone(&ctx.http),
                        is_nsfw,
                        guild_in_cache,
                        gateway,
                    )
                    .await
                {
                    tracing::warn!(error = %error, command = %command.data.name, "oxcord-framework: slash command dispatch failed");
                }
            },
            FullEvent::InteractionCreate {
                interaction: Interaction::Component(component),
            } => {
                self.notify_component(&component);
            },
            FullEvent::InteractionCreate {
                interaction: Interaction::Modal(modal),
            } => {
                self.notify_modal(&modal);
            },
            FullEvent::InteractionCreate {
                interaction: Interaction::Autocomplete(command),
            } => {
                let interaction_id = command.id;
                let token = command.token.clone();
                let gateway: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>> =
                    Some(std::sync::Arc::new(ctx.clone()));
                let suggestions = match self
                    .dispatch_autocomplete(&command, std::sync::Arc::clone(&ctx.http), gateway)
                    .await
                {
                    Ok(suggestions) => suggestions,
                    Err(error) => {
                        tracing::warn!(
                            error = %error,
                            command = %command.data.name,
                            "oxcord-framework: autocomplete dispatch failed"
                        );
                        return;
                    },
                };

                let mut response = CreateAutocompleteResponse::new();
                for suggestion in suggestions.into_iter().take(25) {
                    response = response.add_string_choice(suggestion.clone(), suggestion);
                }
                let result = CreateInteractionResponse::Autocomplete(response)
                    .execute(&*ctx.http, (interaction_id, token.as_str()))
                    .await;
                if let Err(error) = result {
                    tracing::warn!(error = %error, "oxcord-framework: failed to send autocomplete response");
                }
            },
            _ => {},
        }
    }
}
