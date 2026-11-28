use std::sync::Arc;

use tracing::debug;

#[cfg(feature = "gateway")]
use super::event_handler::{EventHandler, RawEventHandler};
use super::{Context, FullEvent};
#[cfg(feature = "cache")]
use crate::cache::{Cache, CacheUpdate};
#[cfg(feature = "framework")]
use crate::framework::Framework;
use crate::internal::tokio::spawn_named;
use crate::model::channel::ChannelType;
use crate::model::event::Event;
use crate::model::guild::Member;
#[cfg(feature = "cache")]
use crate::model::id::GuildId;

#[cfg(feature = "cache")]
macro_rules! if_cache {
    ($e:expr_2021) => {
        $e
    };
}

#[cfg(not(feature = "cache"))]
macro_rules! if_cache {
    ($e:expr) => {
        None
    };
}

#[cfg(feature = "cache")]
macro_rules! update_cache {
    ($cache:ident, $event:ident) => {
        $event.update($cache)
    };
}

#[cfg(not(feature = "cache"))]
macro_rules! update_cache {
    ($cache:ident, $event:ident) => {};
}

pub(crate) fn dispatch_model(
    event: Event,
    context: &Context,
    #[cfg(feature = "framework")] framework: Option<Arc<dyn Framework>>,
    event_handlers: Vec<Arc<dyn EventHandler>>,
    raw_event_handlers: Vec<Arc<dyn RawEventHandler>>,
) {
    for raw_handler in raw_event_handlers {
        let (context, event) = (context.clone(), event.clone());
        tokio::spawn(async move { raw_handler.raw_event(context, event).await });
    }

    let full_events = update_cache_with_event(
        #[cfg(feature = "cache")]
        &context.cache,
        event,
    );

    if let Some(events) = full_events {
        let iter = std::iter::once(events.0).chain(events.1);
        for handler in event_handlers {
            for event in iter.clone() {
                let context = context.clone();
                let handler = Arc::clone(&handler);
                spawn_named(event.snake_case_name(), async move {
                    event.dispatch(context, &*handler).await;
                });
            }
        }

        #[cfg(feature = "framework")]
        if let Some(framework) = framework {
            for event in iter {
                let context = context.clone();
                let framework = Arc::clone(&framework);
                spawn_named("dispatch::framework::dispatch", async move {
                    framework.dispatch(context, event).await;
                });
            }
        }
    }
}

/// Updates the cache with the incoming event data and builds the full event data out of it.
///
/// Can return a secondary [`FullEvent`] for "virtual" events like [`FullEvent::CacheReady`] or
/// [`FullEvent::ShardsReady`]. Secondary events are traditionally dispatched first.
///
/// Can return `None` if an event is unknown.
#[cfg_attr(not(feature = "cache"), allow(unused_mut))]
fn update_cache_with_event(
    #[cfg(feature = "cache")] cache: &Cache,
    event: Event,
) -> Option<(FullEvent, Option<FullEvent>)> {
    let mut extra_event = None;
    let event = match event {
        Event::CommandPermissionsUpdate(event) => FullEvent::CommandPermissionsUpdate {
            permission: event.permission,
        },
        Event::AutoModRuleCreate(event) => FullEvent::AutoModRuleCreate {
            rule: event.rule,
        },
        Event::AutoModRuleUpdate(event) => FullEvent::AutoModRuleUpdate {
            rule: event.rule,
        },
        Event::AutoModRuleDelete(event) => FullEvent::AutoModRuleDelete {
            rule: event.rule,
        },
        Event::AutoModActionExecution(event) => FullEvent::AutoModActionExecution {
            execution: event.execution,
        },
        Event::ChannelCreate(mut event) => {
            update_cache!(cache, event);

            let channel = event.channel;
            if channel.kind == ChannelType::Category {
                FullEvent::CategoryCreate {
                    category: channel,
                }
            } else {
                FullEvent::ChannelCreate {
                    channel,
                }
            }
        },
        Event::ChannelDelete(mut event) => {
            let cached_messages = if_cache!(event.update(cache));

            let channel = event.channel;
            if channel.kind == ChannelType::Category {
                FullEvent::CategoryDelete {
                    category: channel,
                }
            } else {
                FullEvent::ChannelDelete {
                    channel,
                    messages: cached_messages,
                }
            }
        },
        Event::ChannelPinsUpdate(event) => FullEvent::ChannelPinsUpdate {
            pin: event,
        },
        Event::ChannelUpdate(mut event) => {
            let old_channel = if_cache!(event.update(cache));

            FullEvent::ChannelUpdate {
                old: old_channel,
                new: event.channel,
            }
        },
        Event::GuildAuditLogEntryCreate(event) => FullEvent::GuildAuditLogEntryCreate {
            entry: event.entry,
            guild_id: event.guild_id,
        },
        Event::GuildBanAdd(event) => FullEvent::GuildBanAddition {
            guild_id: event.guild_id,
            banned_user: event.user,
        },
        Event::GuildBanRemove(event) => FullEvent::GuildBanRemoval {
            guild_id: event.guild_id,
            unbanned_user: event.user,
        },
        Event::GuildCreate(mut event) => {
            let is_new = if_cache!(Some(!cache.unavailable_guilds.contains(&event.guild.id)));

            update_cache!(cache, event);

            #[cfg(feature = "cache")]
            {
                if cache.unavailable_guilds.is_empty() {
                    cache.unavailable_guilds.shrink_to_fit();

                    let guild_amount =
                        cache.guilds.iter().map(|i| *i.key()).collect::<Vec<GuildId>>();

                    extra_event = Some(FullEvent::CacheReady {
                        guilds: guild_amount,
                    });
                }
            }

            FullEvent::GuildCreate {
                guild: event.guild,
                is_new,
            }
        },
        Event::GuildDelete(mut event) => {
            let full = if_cache!(event.update(cache));

            FullEvent::GuildDelete {
                incomplete: event.guild,
                full,
            }
        },
        Event::GuildEmojisUpdate(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildEmojisUpdate {
                guild_id: event.guild_id,
                current_state: event.emojis,
            }
        },
        Event::GuildIntegrationsUpdate(event) => FullEvent::GuildIntegrationsUpdate {
            guild_id: event.guild_id,
        },
        Event::GuildMemberAdd(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildMemberAddition {
                new_member: event.member,
            }
        },
        Event::GuildMemberRemove(mut event) => {
            let member = if_cache!(event.update(cache));

            FullEvent::GuildMemberRemoval {
                guild_id: event.guild_id,
                user: event.user,
                member_data_if_available: member,
            }
        },
        Event::GuildMemberUpdate(mut event) => {
            let before = if_cache!(event.update(cache));
            let after: Option<Member> = if_cache!({
                let guild = cache.guild(event.guild_id);
                guild.and_then(|g| g.members.get(&event.user.id).cloned())
            });

            FullEvent::GuildMemberUpdate {
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::GuildMembersChunk(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildMembersChunk {
                chunk: event,
            }
        },
        Event::GuildRoleCreate(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildRoleCreate {
                new: event.role,
            }
        },
        Event::GuildRoleDelete(mut event) => {
            let role = if_cache!(event.update(cache));

            FullEvent::GuildRoleDelete {
                guild_id: event.guild_id,
                removed_role_id: event.role_id,
                removed_role_data_if_available: role,
            }
        },
        Event::GuildRoleUpdate(mut event) => {
            let before = if_cache!(event.update(cache));

            FullEvent::GuildRoleUpdate {
                old_data_if_available: before,
                new: event.role,
            }
        },
        Event::GuildStickersUpdate(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildStickersUpdate {
                guild_id: event.guild_id,
                current_state: event.stickers,
            }
        },
        Event::GuildUpdate(event) => {
            let before = if_cache!(cache.guild(event.guild.id).map(|g| g.clone()));

            FullEvent::GuildUpdate {
                old_data_if_available: before,
                new_data: event.guild,
            }
        },
        Event::InviteCreate(event) => FullEvent::InviteCreate {
            data: event,
        },
        Event::InviteDelete(event) => FullEvent::InviteDelete {
            data: event,
        },
        Event::MessageCreate(mut event) => {
            update_cache!(cache, event);

            FullEvent::Message {
                new_message: event.message,
            }
        },
        Event::MessageDeleteBulk(event) => FullEvent::MessageDeleteBulk {
            channel_id: event.channel_id,
            multiple_deleted_messages_ids: event.ids,
            guild_id: event.guild_id,
        },
        Event::MessageDelete(event) => FullEvent::MessageDelete {
            channel_id: event.channel_id,
            deleted_message_id: event.message_id,
            guild_id: event.guild_id,
        },
        Event::MessageUpdate(mut event) => {
            let before = if_cache!(event.update(cache));
            let after = if_cache!(cache.message(event.channel_id, event.id).map(|m| m.clone()));

            FullEvent::MessageUpdate {
                old_if_available: before,
                new: after,
                event,
            }
        },
        Event::PresenceUpdate(mut event) => {
            update_cache!(cache, event);

            FullEvent::PresenceUpdate {
                new_data: event.presence,
            }
        },
        Event::ReactionAdd(event) => FullEvent::ReactionAdd {
            add_reaction: event.reaction,
        },
        Event::ReactionRemove(event) => FullEvent::ReactionRemove {
            removed_reaction: event.reaction,
        },
        Event::ReactionRemoveAll(event) => FullEvent::ReactionRemoveAll {
            channel_id: event.channel_id,
            removed_from_message_id: event.message_id,
        },
        Event::ReactionRemoveEmoji(event) => FullEvent::ReactionRemoveEmoji {
            removed_reactions: event.reaction,
        },
        Event::Ready(mut event) => {
            update_cache!(cache, event);

            #[cfg(feature = "cache")]
            {
                let mut shards = cache.shard_data.write();
                if shards.connected.len() as u32 == shards.total && !shards.has_sent_shards_ready {
                    shards.has_sent_shards_ready = true;
                    let total = shards.total;
                    drop(shards);

                    extra_event = Some(FullEvent::ShardsReady {
                        total_shards: total,
                    });
                }
            }

            FullEvent::Ready {
                data_about_bot: event.ready,
            }
        },
        Event::Resumed(event) => FullEvent::Resume {
            event,
        },
        Event::SoundboardSounds(event) => FullEvent::SoundboardSounds {
            event,
        },
        Event::SoundboardSoundCreate(event) => FullEvent::SoundboardSoundCreate {
            event,
        },
        Event::SoundboardSoundUpdate(event) => FullEvent::SoundboardSoundUpdate {
            event,
        },
        Event::SoundboardSoundsUpdate(event) => FullEvent::SoundboardSoundsUpdate {
            event,
        },
        Event::SoundboardSoundDelete(event) => FullEvent::SoundboardSoundDelete {
            event,
        },
        Event::TypingStart(event) => FullEvent::TypingStart {
            event,
        },
        Event::Unknown(event) => {
            debug!("An unknown event was received: {event:?}");
            return None;
        },
        Event::UserUpdate(mut event) => {
            let before = if_cache!(event.update(cache));

            FullEvent::UserUpdate {
                old_data: before,
                new: event.current_user,
            }
        },
        Event::VoiceServerUpdate(event) => FullEvent::VoiceServerUpdate {
            event,
        },
        Event::VoiceStateUpdate(mut event) => {
            let before = if_cache!(event.update(cache));

            FullEvent::VoiceStateUpdate {
                old: before,
                new: event.voice_state,
            }
        },
        Event::VoiceChannelStatusUpdate(mut event) => {
            let old = if_cache!(event.update(cache));

            FullEvent::VoiceChannelStatusUpdate {
                old,
                status: event.status,
                id: event.id,
                guild_id: event.guild_id,
            }
        },
        Event::VoiceChannelEffectSend(event) => FullEvent::VoiceChannelEffectSend {
            event,
        },

        Event::WebhookUpdate(event) => FullEvent::WebhookUpdate {
            guild_id: event.guild_id,
            belongs_to_channel_id: event.channel_id,
        },
        Event::InteractionCreate(event) => FullEvent::InteractionCreate {
            interaction: event.interaction,
        },
        Event::IntegrationCreate(event) => FullEvent::IntegrationCreate {
            integration: event.integration,
        },
        Event::IntegrationUpdate(event) => FullEvent::IntegrationUpdate {
            integration: event.integration,
        },
        Event::IntegrationDelete(event) => FullEvent::IntegrationDelete {
            integration_id: event.id,
            guild_id: event.guild_id,
            application_id: event.application_id,
        },
        Event::StageInstanceCreate(event) => FullEvent::StageInstanceCreate {
            stage_instance: event.stage_instance,
        },
        Event::StageInstanceUpdate(event) => FullEvent::StageInstanceUpdate {
            stage_instance: event.stage_instance,
        },
        Event::StageInstanceDelete(event) => FullEvent::StageInstanceDelete {
            stage_instance: event.stage_instance,
        },
        Event::ThreadCreate(mut event) => {
            update_cache!(cache, event);

            FullEvent::ThreadCreate {
                thread: event.thread,
            }
        },
        Event::ThreadUpdate(mut event) => {
            let old = if_cache!(event.update(cache));

            FullEvent::ThreadUpdate {
                old,
                new: event.thread,
            }
        },
        Event::ThreadDelete(mut event) => {
            let full_thread_data = if_cache!(event.update(cache));

            FullEvent::ThreadDelete {
                thread: event.thread,
                full_thread_data,
            }
        },
        Event::ThreadListSync(event) => FullEvent::ThreadListSync {
            thread_list_sync: event,
        },
        Event::ThreadMemberUpdate(event) => FullEvent::ThreadMemberUpdate {
            thread_member: event.member,
        },
        Event::ThreadMembersUpdate(event) => FullEvent::ThreadMembersUpdate {
            thread_members_update: event,
        },
        Event::GuildScheduledEventCreate(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildScheduledEventCreate {
                event: event.event,
            }
        },
        Event::GuildScheduledEventUpdate(mut event) => {
            let old = if_cache!(event.update(cache));

            extra_event = Some(FullEvent::GuildScheduledEventUpdateWithOld {
                old_if_available: old,
                new: event.event.clone(),
            });

            FullEvent::GuildScheduledEventUpdate {
                event: event.event,
            }
        },
        Event::GuildScheduledEventDelete(mut event) => {
            update_cache!(cache, event);

            FullEvent::GuildScheduledEventDelete {
                event: event.event,
            }
        },
        Event::GuildScheduledEventUserAdd(event) => FullEvent::GuildScheduledEventUserAdd {
            subscribed: event,
        },
        Event::GuildScheduledEventUserRemove(event) => FullEvent::GuildScheduledEventUserRemove {
            unsubscribed: event,
        },
        Event::EntitlementCreate(event) => FullEvent::EntitlementCreate {
            entitlement: event.entitlement,
        },
        Event::EntitlementUpdate(event) => FullEvent::EntitlementUpdate {
            entitlement: event.entitlement,
        },
        Event::EntitlementDelete(event) => FullEvent::EntitlementDelete {
            entitlement: event.entitlement,
        },
        Event::SubscriptionCreate(event) => FullEvent::SubscriptionCreate {
            subscription: event.subscription,
        },
        Event::SubscriptionUpdate(event) => FullEvent::SubscriptionUpdate {
            subscription: event.subscription,
        },
        Event::SubscriptionDelete(event) => FullEvent::SubscriptionDelete {
            subscription: event.subscription,
        },
        Event::MessagePollVoteAdd(event) => FullEvent::MessagePollVoteAdd {
            event,
        },
        Event::MessagePollVoteRemove(event) => FullEvent::MessagePollVoteRemove {
            event,
        },
        _ => return None,
    };

    Some((event, extra_event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::id::GuildId;

    fn run(event: Event) -> (FullEvent, Option<FullEvent>) {
        #[cfg(feature = "cache")]
        let cache = Cache::default();
        update_cache_with_event(
            #[cfg(feature = "cache")]
            &cache,
            event,
        )
        .expect("known event")
    }

    fn scheduled_event(id: u64, name: &str) -> serde_json::Value {
        serde_json::json!({
            "id": id.to_string(),
            "guild_id": "1",
            "name": name,
            "scheduled_start_time": "2026-01-01T00:00:00.000000+00:00",
            "privacy_level": 2,
            "status": 1,
            "entity_type": 3,
        })
    }

    fn update_event(id: u64, name: &str) -> Event {
        Event::GuildScheduledEventUpdate(serde_json::from_value(scheduled_event(id, name)).unwrap())
    }

    #[test]
    fn scheduled_event_update_without_cached_guild_has_no_old() {
        let (primary, extra) = run(update_event(5, "new"));
        assert!(matches!(primary, FullEvent::GuildScheduledEventUpdate { .. }));
        match extra {
            Some(FullEvent::GuildScheduledEventUpdateWithOld {
                old_if_available,
                new,
            }) => {
                assert!(old_if_available.is_none());
                assert_eq!(new.name, "new");
            },
            other => panic!("unexpected extra event: {other:?}"),
        }
    }

    #[cfg(feature = "cache")]
    #[test]
    fn scheduled_event_cache_follows_create_update_delete() {
        use crate::model::guild::Guild;

        let cache = Cache::default();
        cache.guilds.insert(GuildId::new(1), Guild {
            id: GuildId::new(1),
            ..Default::default()
        });
        let events = |cache: &Cache| cache.guild(GuildId::new(1)).unwrap().scheduled_events.clone();

        let create = Event::GuildScheduledEventCreate(
            serde_json::from_value(scheduled_event(5, "old")).unwrap(),
        );
        update_cache_with_event(&cache, create).unwrap();
        assert_eq!(events(&cache).len(), 1);

        let (_, extra) = update_cache_with_event(&cache, update_event(5, "new")).unwrap();
        let Some(FullEvent::GuildScheduledEventUpdateWithOld {
            old_if_available,
            new,
        }) = extra
        else {
            panic!("missing extra event");
        };
        assert_eq!(old_if_available.expect("cached").name, "old");
        assert_eq!(new.name, "new");
        assert_eq!(events(&cache)[0].name, "new");

        let delete = Event::GuildScheduledEventDelete(
            serde_json::from_value(scheduled_event(5, "new")).unwrap(),
        );
        update_cache_with_event(&cache, delete).unwrap();
        assert!(events(&cache).is_empty());
    }
}
