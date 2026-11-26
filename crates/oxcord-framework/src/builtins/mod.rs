//! Helper dan command bawaan siap pakai (setara `poise::builtins`).

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::time::Duration;

use oxcord_builders::{
    Builder, CreateActionRow, CreateButton, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage,
};
use oxcord_model::application::ButtonStyle;
use oxcord_model::id::GuildId;

use crate::command::{Command, CommandKind};
use crate::context::{Context, ErrorContext, Reply};
use crate::error::FrameworkError;

/// Tampilkan daftar halaman dengan tombol navigasi Previous / Next (setara `poise::builtins::paginate`).
///
/// Tombol aktif selama 2 menit sejak interaksi terakhir; setelah timeout tombol otomatis dihapus.
///
/// # Errors
///
/// Mengembalikan [`FrameworkError::Http`] jika pengiriman atau update pesan ke Discord gagal.
pub async fn paginate(ctx: Context<'_>, pages: &[&str]) -> Result<(), FrameworkError> {
    if pages.is_empty() {
        return Ok(());
    }
    if pages.len() == 1 {
        ctx.say(pages[0]).await?;
        return Ok(());
    }

    let ctx_id = ctx.created_at().timestamp_millis();
    let prev_id = format!("{ctx_id}_prev");
    let next_id = format!("{ctx_id}_next");

    let build_components = |page: usize, disabled: bool| {
        vec![CreateActionRow::Buttons(vec![
            CreateButton::new(&prev_id)
                .label("◀")
                .style(ButtonStyle::Secondary)
                .disabled(disabled || page == 0),
            CreateButton::new(&next_id)
                .label("▶")
                .style(ButtonStyle::Secondary)
                .disabled(disabled || page + 1 >= pages.len()),
        ])]
    };

    let mut current_page = 0usize;
    let handle = ctx
        .send(
            Reply::new()
                .embed(CreateEmbed::new().description(pages[current_page]).footer(
                    oxcord_builders::CreateEmbedFooter::new(format!(
                        "Page {}/{}",
                        current_page + 1,
                        pages.len()
                    )),
                ))
                .components(build_components(current_page, false)),
        )
        .await?;

    let Some(mut rx) = ctx.subscribe_components() else {
        return Ok(());
    };

    let timeout = Duration::from_secs(120);
    loop {
        let received = tokio::time::timeout(timeout, rx.recv()).await;
        match received {
            Ok(Ok(interaction)) => {
                if interaction.user.id != ctx.author.id {
                    continue;
                }
                if interaction.data.custom_id == prev_id {
                    current_page = current_page.saturating_sub(1);
                } else if interaction.data.custom_id == next_id {
                    if current_page + 1 < pages.len() {
                        current_page += 1;
                    }
                } else {
                    continue;
                }

                let update = CreateInteractionResponseMessage::new()
                    .embed(CreateEmbed::new().description(pages[current_page]).footer(
                        oxcord_builders::CreateEmbedFooter::new(format!(
                            "Page {}/{}",
                            current_page + 1,
                            pages.len()
                        )),
                    ))
                    .components(build_components(current_page, false));
                let _ = CreateInteractionResponse::UpdateMessage(update)
                    .execute(ctx.http(), (interaction.id, interaction.token.as_str()))
                    .await;
            },
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => {},
            _ => break,
        }
    }

    let _ = handle
        .edit(
            Reply::new()
                .embed(CreateEmbed::new().description(pages[current_page]).footer(
                    oxcord_builders::CreateEmbedFooter::new(format!(
                        "Page {}/{}",
                        current_page + 1,
                        pages.len()
                    )),
                ))
                .components(vec![]),
        )
        .await;
    Ok(())
}

/// Daftarkan daftar [`Command`] secara global ke Discord (bulk overwrite).
///
/// # Errors
///
/// Mengembalikan [`FrameworkError::Http`] jika request ke Discord gagal.
pub async fn register_globally(
    ctx: Context<'_>,
    commands: &[Command],
) -> Result<(), FrameworkError> {
    let fw = crate::framework::Framework::builder()
        .options(crate::framework::FrameworkOptions {
            commands: commands.to_vec(),
            ..Default::default()
        })
        .build();
    fw.register_global_commands(ctx.http()).await
}

/// Daftarkan daftar [`Command`] ke satu guild tertentu (bulk overwrite).
///
/// # Errors
///
/// Mengembalikan [`FrameworkError::Http`] jika request ke Discord gagal.
pub async fn register_in_guild(
    ctx: Context<'_>,
    commands: &[Command],
    guild_id: GuildId,
) -> Result<(), FrameworkError> {
    let fw = crate::framework::Framework::builder()
        .options(crate::framework::FrameworkOptions {
            commands: commands.to_vec(),
            ..Default::default()
        })
        .build();
    fw.register_guild_commands(ctx.http(), guild_id).await
}

/// Tampilkan UI tombol interaktif untuk register/unregister slash command (global atau per-guild),
/// setara `poise::builtins::register_application_commands_buttons`.
///
/// # Errors
///
/// Mengembalikan [`FrameworkError`] jika pengiriman pesan atau registrasi ke Discord gagal.
pub async fn register_application_commands_buttons(
    ctx: Context<'_>,
    commands: &[Command],
) -> Result<(), FrameworkError> {
    let ctx_id = ctx.created_at().timestamp_millis();
    let reg_guild_id = format!("{ctx_id}_reg_guild");
    let unreg_guild_id = format!("{ctx_id}_unreg_guild");
    let reg_global_id = format!("{ctx_id}_reg_global");
    let unreg_global_id = format!("{ctx_id}_unreg_global");

    let mut buttons = vec![
        CreateButton::new(&reg_global_id).label("Register globally").style(ButtonStyle::Primary),
        CreateButton::new(&unreg_global_id).label("Unregister globally").style(ButtonStyle::Danger),
    ];
    if ctx.guild_id.is_some() {
        buttons.push(
            CreateButton::new(&reg_guild_id).label("Register in guild").style(ButtonStyle::Primary),
        );
        buttons.push(
            CreateButton::new(&unreg_guild_id)
                .label("Unregister in guild")
                .style(ButtonStyle::Danger),
        );
    }

    let handle = ctx
        .send(
            Reply::new()
                .content("Pilih aksi registrasi slash command:")
                .components(vec![CreateActionRow::Buttons(buttons)]),
        )
        .await?;

    let Some(mut rx) = ctx.subscribe_components() else {
        return Ok(());
    };

    let timeout = Duration::from_secs(60);
    while let Ok(Ok(interaction)) = tokio::time::timeout(timeout, rx.recv()).await {
        if interaction.user.id != ctx.author.id {
            continue;
        }
        let cid = interaction.data.custom_id.as_str();
        let status = if cid == reg_global_id {
            register_globally(ctx.clone(), commands).await.map(|()| "Registered global commands!")
        } else if cid == unreg_global_id {
            ctx.http()
                .create_global_commands(&Vec::<oxcord_builders::CreateCommand>::new())
                .await
                .map(|_| "Unregistered global commands!")
                .map_err(|e| FrameworkError::Http(e.to_string()))
        } else if cid == reg_guild_id {
            if let Some(gid) = ctx.guild_id {
                register_in_guild(ctx.clone(), commands, gid)
                    .await
                    .map(|()| "Registered guild commands!")
            } else {
                Ok("Not in a guild")
            }
        } else if cid == unreg_guild_id {
            if let Some(gid) = ctx.guild_id {
                ctx.http()
                    .create_guild_commands(gid, &Vec::<oxcord_builders::CreateCommand>::new())
                    .await
                    .map(|_| "Unregistered guild commands!")
                    .map_err(|e| FrameworkError::Http(e.to_string()))
            } else {
                Ok("Not in a guild")
            }
        } else {
            continue;
        };

        let msg = match status {
            Ok(s) => s.to_owned(),
            Err(e) => format!("Failed: {e}"),
        };
        let _ = CreateInteractionResponse::UpdateMessage(
            CreateInteractionResponseMessage::new().content(msg).components(vec![]),
        )
        .execute(ctx.http(), (interaction.id, interaction.token.as_str()))
        .await;
        return Ok(());
    }

    let _ = handle.edit(Reply::new().content("Timed out.").components(vec![])).await;
    Ok(())
}

/// Konfigurasi opsional untuk [`help`].
#[derive(Debug, Clone, Default)]
pub struct HelpConfiguration<'a> {
    /// Teks tambahan di bagian bawah menu help.
    pub extra_text_at_bottom: Option<&'a str>,
    /// Apakah command dikirim sebagai ephemeral (kalau slash).
    pub ephemeral: bool,
    /// Tampilkan subcommand di daftar utama.
    pub show_subcommands: bool,
}

/// Generate pesan help otomatis dari daftar [`Command`] (memanfaatkan `name`, `description`,
/// `category`, `help_text`, `hide_in_help`, `aliases`, `subcommands`).
///
/// # Errors
///
/// Mengembalikan [`FrameworkError::Http`] jika pengiriman pesan gagal.
pub async fn help(
    ctx: Context<'_>,
    commands: &[Command],
    command_name: Option<&str>,
    config: HelpConfiguration<'_>,
) -> Result<(), FrameworkError> {
    if let Some(target_name) = command_name {
        let Some(cmd) = commands.iter().find(|c| {
            !c.hide_in_help && (c.name == target_name || c.aliases.iter().any(|a| a == target_name))
        }) else {
            ctx.send(
                Reply::new()
                    .content(format!("Command `{target_name}` tidak ditemukan."))
                    .ephemeral(config.ephemeral),
            )
            .await?;
            return Ok(());
        };

        let mut text = cmd.help_text.clone().unwrap_or_else(|| cmd.description.clone());
        if !cmd.aliases.is_empty() {
            let _ = write!(text, "\n\n**Aliases:** {}", cmd.aliases.join(", "));
        }
        if !cmd.subcommands.is_empty() {
            text.push_str("\n\n**Subcommands:**\n");
            for sub in &cmd.subcommands {
                if !sub.hide_in_help {
                    let _ = writeln!(text, "• `{}` — {}", sub.name, sub.description);
                }
            }
        }
        let embed = CreateEmbed::new().title(format!("Help: {}", cmd.name)).description(text);
        ctx.send(Reply::new().embed(embed).ephemeral(config.ephemeral)).await?;
        return Ok(());
    }

    let mut by_category: BTreeMap<String, Vec<&Command>> = BTreeMap::new();
    for cmd in commands {
        if cmd.hide_in_help {
            continue;
        }
        if matches!(cmd.kind, CommandKind::Slash)
            && cmd.handler.is_none()
            && cmd.subcommands.is_empty()
            && cmd.context_menu_action.is_none()
        {
            continue;
        }
        let cat = cmd.category.0.clone().unwrap_or_else(|| "Commands".to_owned());
        by_category.entry(cat).or_default().push(cmd);
    }

    let mut embed = CreateEmbed::new().title("Help Menu");
    for (cat, cmds) in by_category {
        let mut lines = String::new();
        for cmd in cmds {
            let _ = writeln!(lines, "**`{}`** — {}", cmd.name, cmd.description);
            if config.show_subcommands {
                for sub in &cmd.subcommands {
                    if !sub.hide_in_help {
                        let _ = writeln!(lines, "  └ `{}` — {}", sub.name, sub.description);
                    }
                }
            }
        }
        if !lines.is_empty() {
            embed = embed.field(cat, lines, false);
        }
    }
    if let Some(bottom) = config.extra_text_at_bottom {
        embed = embed.footer(oxcord_builders::CreateEmbedFooter::new(bottom));
    }

    ctx.send(Reply::new().embed(embed).ephemeral(config.ephemeral)).await?;
    Ok(())
}

/// Handler error bawaan yang rapi untuk dipasang di `FrameworkOptions::on_error`.
pub async fn on_error(error: FrameworkError, ctx: ErrorContext) {
    let reply_text = match &error {
        FrameworkError::Cooldown { remaining_secs } => {
            format!("Tunggu {remaining_secs:.1} detik lagi sebelum pakai command ini.")
        },
        FrameworkError::GuildOnly => "Command ini cuma bisa dipakai di dalam server.".to_owned(),
        FrameworkError::DmOnly => "Command ini cuma bisa dipakai di DM.".to_owned(),
        FrameworkError::NsfwOnly => "Command ini cuma bisa dipakai di channel NSFW.".to_owned(),
        FrameworkError::OwnerOnly => "Command ini khusus untuk owner bot.".to_owned(),
        FrameworkError::MissingUserPermissions { missing } => {
            format!("Kamu kekurangan permission: `{missing:?}`")
        },
        FrameworkError::MissingBotPermissions { missing } => {
            format!("Bot kekurangan permission: `{missing:?}`")
        },
        FrameworkError::SubcommandRequired => "Silakan pilih subcommand yang valid.".to_owned(),
        FrameworkError::CommandCheckFailed(_) => {
            "Kamu tidak memenuhi syarat untuk menjalankan command ini.".to_owned()
        },
        other => {
            tracing::warn!(command = %ctx.command_name, error = %other, "oxcord-framework error");
            format!("Terjadi error: {other}")
        },
    };
    let _ = ctx.ctx().send(Reply::new().content(reply_text).ephemeral(true)).await;
}

/// Helper autocomplete yang mengembalikan daftar nama command yang cocok dengan `partial`.
#[must_use]
pub fn autocomplete_command(commands: &[Command], partial: &str) -> Vec<String> {
    let needle = partial.to_lowercase();
    commands
        .iter()
        .filter(|c| !c.hide_in_help && c.name.to_lowercase().starts_with(&needle))
        .take(25)
        .map(|c| c.name.clone())
        .collect()
}

/// Tampilkan daftar server (guild) tempat bot berada saat ini (setara `poise::builtins::servers`).
///
/// # Errors
///
/// Mengembalikan [`FrameworkError::Http`] jika request ke Discord gagal.
pub async fn servers(ctx: Context<'_>) -> Result<(), FrameworkError> {
    let guilds = ctx
        .http()
        .get_guilds(None, Some(100))
        .await
        .map_err(|e| FrameworkError::Http(e.to_string()))?;

    let mut desc = String::new();
    for g in &guilds {
        let _ = writeln!(desc, "• **{}** (`{}`)", g.name, g.id);
    }
    if desc.is_empty() {
        "Bot belum berada di server mana pun.".clone_into(&mut desc);
    }
    ctx.send(
        Reply::new().embed(
            CreateEmbed::new().title(format!("Servers ({})", guilds.len())).description(desc),
        ),
    )
    .await?;
    Ok(())
}
