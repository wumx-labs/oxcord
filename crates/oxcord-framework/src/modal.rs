//! Dukungan form interaktif Discord (Modal), setara `modal.rs` di poise.

use std::time::Duration;

use oxcord_builders::{Builder, CreateInteractionResponse, CreateModal};
use oxcord_model::application::{ComponentInteraction, ModalInteraction, ModalInteractionData};

use crate::context::Context;
use crate::error::FrameworkError;

/// Trait untuk struct yang merepresentasikan form modal Discord.
pub trait Modal: Sized {
    /// Bangun payload [`CreateModal`] dengan `custom_id` unik dan nilai default opsional.
    fn create(defaults: Option<Self>, custom_id: String) -> CreateModal;

    /// Parse data jawaban user dari [`ModalInteractionData`] kembali menjadi `Self`.
    ///
    /// # Errors
    ///
    /// Mengembalikan [`FrameworkError::ArgumentParse`] atau [`FrameworkError::Other`] jika field tidak valid.
    fn parse(data: ModalInteractionData) -> Result<Self, FrameworkError>;
}

/// Helper internal untuk mengirim modal dan menunggu submit-nya di `InteractionBus`.
async fn execute_modal_generic<M: Modal>(
    ctx: &Context<'_>,
    interaction_id: oxcord_model::id::InteractionId,
    token: &str,
    defaults: Option<M>,
    timeout: Duration,
) -> Result<Option<M>, FrameworkError> {
    let custom_id = format!("oxcord_modal_{}_{}", interaction_id.get(), ctx.author.id.get());
    let modal = M::create(defaults, custom_id.clone());

    let mut rx = ctx.subscribe_modals().ok_or_else(|| {
        FrameworkError::Other("interaction bus is not available for modals".to_owned())
    })?;

    CreateInteractionResponse::Modal(modal)
        .execute(ctx.http(), (interaction_id, token))
        .await
        .map_err(|e| FrameworkError::Http(e.to_string()))?;

    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            () = &mut deadline => return Ok(None),
            msg = rx.recv() => {
                match msg {
                    Ok(interaction) if interaction.data.custom_id == custom_id && interaction.user.id == ctx.author.id => {
                        let parsed = M::parse(interaction.data)?;
                        return Ok(Some(parsed));
                    },
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return Ok(None),
                }
            }
        }
    }
}

/// Tampilkan modal untuk slash command saat ini dan tunggu user men-submit.
///
/// Mengembalikan `Ok(None)` jika user tidak men-submit sebelum `timeout` (atau menutup modal).
///
/// # Errors
///
/// Mengembalikan [`FrameworkError`] jika bukan slash invocation, HTTP gagal, atau parsing modal gagal.
pub async fn execute_modal<M: Modal>(
    ctx: Context<'_>,
    defaults: Option<M>,
    timeout: Duration,
) -> Result<Option<M>, FrameworkError> {
    let interaction_id = ctx.http.interaction_id.ok_or_else(|| {
        FrameworkError::Other("execute_modal only works on slash interactions".to_owned())
    })?;
    let token = match ctx.invocation {
        crate::context::InvocationData::Slash {
            token, ..
        } => token.as_str(),
        crate::context::InvocationData::Prefix {
            ..
        } => {
            return Err(FrameworkError::Other(
                "execute_modal only works on slash interactions".to_owned(),
            ));
        },
    };
    execute_modal_generic(&ctx, interaction_id, token, defaults, timeout).await
}

/// Tampilkan modal sebagai respons dari sebuah [`ComponentInteraction`] (tombol / select menu).
///
/// # Errors
///
/// Mengembalikan [`FrameworkError`] jika HTTP gagal atau parsing modal gagal.
pub async fn execute_modal_on_component_interaction<M: Modal>(
    ctx: Context<'_>,
    interaction: &ComponentInteraction,
    defaults: Option<M>,
    timeout: Duration,
) -> Result<Option<M>, FrameworkError> {
    let custom_id = format!("oxcord_modal_{}_{}", interaction.id.get(), interaction.user.id.get());
    let modal = M::create(defaults, custom_id.clone());

    let mut rx = ctx.subscribe_modals().ok_or_else(|| {
        FrameworkError::Other("interaction bus is not available for modals".to_owned())
    })?;

    CreateInteractionResponse::Modal(modal)
        .execute(ctx.http(), (interaction.id, interaction.token.as_str()))
        .await
        .map_err(|e| FrameworkError::Http(e.to_string()))?;

    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            () = &mut deadline => return Ok(None),
            msg = rx.recv() => {
                match msg {
                    Ok(modal_int) if modal_int.data.custom_id == custom_id && modal_int.user.id == interaction.user.id => {
                        let _ = ModalInteraction::clone(&modal_int);
                        let parsed = M::parse(modal_int.data)?;
                        return Ok(Some(parsed));
                    },
                    Ok(_) | Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {},
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return Ok(None),
                }
            }
        }
    }
}
