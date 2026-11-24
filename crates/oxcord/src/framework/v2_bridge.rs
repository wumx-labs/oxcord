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

#[async_trait]
impl Framework for oxcord_framework::Framework {
    async fn dispatch(&self, ctx: Context, event: FullEvent) {
        match event {
            FullEvent::Ready {
                ..
            } => {
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
                if new_message.author.bot {
                    return;
                }
                let Some(result) = self
                    .dispatch_message(
                        &new_message.author,
                        new_message.channel_id,
                        new_message.guild_id,
                        &new_message.content,
                        ctx.http.clone(),
                    )
                    .await
                else {
                    // No prefix configured, message didn't start with it, or
                    // no command name followed it — not every message is a
                    // command, this is the normal case.
                    return;
                };
                if let Err(error) = result {
                    tracing::warn!(error = %error, "oxcord-framework: prefix command dispatch failed");
                }
            },
            FullEvent::InteractionCreate {
                interaction: Interaction::Command(command),
            } => {
                if let Err(error) = self.dispatch_slash(&command, ctx.http.clone()).await {
                    tracing::warn!(error = %error, command = %command.data.name, "oxcord-framework: slash command dispatch failed");
                }
            },
            FullEvent::InteractionCreate {
                interaction: Interaction::Autocomplete(command),
            } => {
                let interaction_id = command.id;
                let token = command.token.clone();
                let suggestions = match self.dispatch_autocomplete(&command, ctx.http.clone()).await {
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
