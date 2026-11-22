# API Coverage Matrix

This document provides a comprehensive, transparent breakdown of Discord API v10 coverage in **OXCORD v0.1.0**.

---

## ⚡ Gateway Events Coverage

Every event dispatched by Discord Gateway v10 is typed in `oxcord-model::event::Event` and dispatched to handlers implementing `EventHandler` or `RawEventHandler`.

| Discord Dispatch Event | Rust Model Type | Gateway Intent | Support Status |
|---|---|---|:---:|
| `READY` | `ReadyEvent` | None | ✅ Supported |
| `RESUMED` | `ResumedEvent` | None | ✅ Supported |
| `COMMAND_PERMISSIONS_UPDATE` | `CommandPermissionsUpdateEvent` | None | ✅ Supported |
| `AUTO_MODERATION_RULE_CREATE` | `AutoModRuleCreateEvent` | `AUTO_MODERATION_CONFIGURATION` | ✅ Supported |
| `AUTO_MODERATION_RULE_UPDATE` | `AutoModRuleUpdateEvent` | `AUTO_MODERATION_CONFIGURATION` | ✅ Supported |
| `AUTO_MODERATION_RULE_DELETE` | `AutoModRuleDeleteEvent` | `AUTO_MODERATION_CONFIGURATION` | ✅ Supported |
| `AUTO_MODERATION_ACTION_EXECUTION` | `AutoModActionExecutionEvent` | `AUTO_MODERATION_EXECUTION` | ✅ Supported |
| `CHANNEL_CREATE` | `ChannelCreateEvent` | `GUILDS` | ✅ Supported |
| `CHANNEL_UPDATE` | `ChannelUpdateEvent` | `GUILDS` | ✅ Supported |
| `CHANNEL_DELETE` | `ChannelDeleteEvent` | `GUILDS` | ✅ Supported |
| `CHANNEL_PINS_UPDATE` | `ChannelPinsUpdateEvent` | `GUILDS`, `DIRECT_MESSAGES` | ✅ Supported |
| `THREAD_CREATE` | `ThreadCreateEvent` | `GUILDS` | ✅ Supported |
| `THREAD_UPDATE` | `ThreadUpdateEvent` | `GUILDS` | ✅ Supported |
| `THREAD_DELETE` | `ThreadDeleteEvent` | `GUILDS` | ✅ Supported |
| `THREAD_LIST_SYNC` | `ThreadListSyncEvent` | `GUILDS` | ✅ Supported |
| `THREAD_MEMBER_UPDATE` | `ThreadMemberUpdateEvent` | `GUILDS`, `GUILD_MEMBERS` | ✅ Supported |
| `THREAD_MEMBERS_UPDATE` | `ThreadMembersUpdateEvent` | `GUILDS`, `GUILD_MEMBERS` | ✅ Supported |
| `GUILD_CREATE` | `GuildCreateEvent` | `GUILDS` | ✅ Supported |
| `GUILD_UPDATE` | `GuildUpdateEvent` | `GUILDS` | ✅ Supported |
| `GUILD_DELETE` | `GuildDeleteEvent` | `GUILDS` | ✅ Supported |
| `GUILD_BAN_ADD` | `GuildBanAddEvent` | `GUILD_MODERATION` | ✅ Supported |
| `GUILD_BAN_REMOVE` | `GuildBanRemoveEvent` | `GUILD_MODERATION` | ✅ Supported |
| `GUILD_EMOJIS_UPDATE` | `GuildEmojisUpdateEvent` | `GUILD_EMOJIS_AND_STICKERS` | ✅ Supported |
| `GUILD_STICKERS_UPDATE` | `GuildStickersUpdateEvent` | `GUILD_EMOJIS_AND_STICKERS` | ✅ Supported |
| `GUILD_INTEGRATIONS_UPDATE` | `GuildIntegrationsUpdateEvent` | `GUILD_INTEGRATIONS` | ✅ Supported |
| `GUILD_MEMBER_ADD` | `GuildMemberAddEvent` | `GUILD_MEMBERS` | ✅ Supported |
| `GUILD_MEMBER_REMOVE` | `GuildMemberRemoveEvent` | `GUILD_MEMBERS` | ✅ Supported |
| `GUILD_MEMBER_UPDATE` | `GuildMemberUpdateEvent` | `GUILD_MEMBERS` | ✅ Supported |
| `GUILD_MEMBERS_CHUNK` | `GuildMembersChunkEvent` | `GUILD_MEMBERS` | ✅ Supported |
| `GUILD_ROLE_CREATE` | `GuildRoleCreateEvent` | `GUILDS` | ✅ Supported |
| `GUILD_ROLE_UPDATE` | `GuildRoleUpdateEvent` | `GUILDS` | ✅ Supported |
| `GUILD_ROLE_DELETE` | `GuildRoleDeleteEvent` | `GUILDS` | ✅ Supported |
| `INTEGRATION_CREATE` | `IntegrationCreateEvent` | `GUILD_INTEGRATIONS` | ✅ Supported |
| `INTEGRATION_UPDATE` | `IntegrationUpdateEvent` | `GUILD_INTEGRATIONS` | ✅ Supported |
| `INTEGRATION_DELETE` | `IntegrationDeleteEvent` | `GUILD_INTEGRATIONS` | ✅ Supported |
| `INTERACTION_CREATE` | `InteractionCreateEvent` | None | ✅ Supported |
| `INVITE_CREATE` | `InviteCreateEvent` | `GUILD_INVITES` | ✅ Supported |
| `INVITE_DELETE` | `InviteDeleteEvent` | `GUILD_INVITES` | ✅ Supported |
| `MESSAGE_CREATE` | `MessageCreateEvent` | `GUILD_MESSAGES`, `DIRECT_MESSAGES` | ✅ Supported |
| `MESSAGE_UPDATE` | `MessageUpdateEvent` | `GUILD_MESSAGES`, `DIRECT_MESSAGES` | ✅ Supported |
| `MESSAGE_DELETE` | `MessageDeleteEvent` | `GUILD_MESSAGES`, `DIRECT_MESSAGES` | ✅ Supported |
| `MESSAGE_DELETE_BULK` | `MessageDeleteBulkEvent` | `GUILD_MESSAGES` | ✅ Supported |
| `MESSAGE_REACTION_ADD` | `ReactionAddEvent` | `GUILD_MESSAGE_REACTIONS`, `DIRECT_MESSAGES` | ✅ Supported |
| `MESSAGE_REACTION_REMOVE` | `ReactionRemoveEvent` | `GUILD_MESSAGE_REACTIONS`, `DIRECT_MESSAGES` | ✅ Supported |
| `MESSAGE_REACTION_REMOVE_ALL` | `ReactionRemoveAllEvent` | `GUILD_MESSAGE_REACTIONS` | ✅ Supported |
| `MESSAGE_REACTION_REMOVE_EMOJI` | `ReactionRemoveEmojiEvent` | `GUILD_MESSAGE_REACTIONS` | ✅ Supported |
| `PRESENCE_UPDATE` | `PresenceUpdateEvent` | `GUILD_PRESENCES` | ✅ Supported |
| `STAGE_INSTANCE_CREATE` | `StageInstanceCreateEvent` | `GUILDS` | ✅ Supported |
| `STAGE_INSTANCE_UPDATE` | `StageInstanceUpdateEvent` | `GUILDS` | ✅ Supported |
| `STAGE_INSTANCE_DELETE` | `StageInstanceDeleteEvent` | `GUILDS` | ✅ Supported |
| `TYPING_START` | `TypingStartEvent` | `GUILD_MESSAGE_TYPING`, `DIRECT_MESSAGES` | ✅ Supported |
| `USER_UPDATE` | `UserUpdateEvent` | None | ✅ Supported |
| `VOICE_STATE_UPDATE` | `VoiceStateUpdateEvent` | `GUILD_VOICE_STATES` | ✅ Supported |
| `VOICE_SERVER_UPDATE` | `VoiceServerUpdateEvent` | `GUILD_VOICE_STATES` | ✅ Supported |
| `VOICE_CHANNEL_STATUS_UPDATE` | `VoiceChannelStatusUpdateEvent` | `GUILDS` | ✅ Supported |
| `WEBHOOKS_UPDATE` | `WebhookUpdateEvent` | `GUILD_WEBHOOKS` | ✅ Supported |
| `ENTITLEMENT_CREATE` | `EntitlementCreateEvent` | None | ✅ Supported |
| `ENTITLEMENT_UPDATE` | `EntitlementUpdateEvent` | None | ✅ Supported |
| `ENTITLEMENT_DELETE` | `EntitlementDeleteEvent` | None | ✅ Supported |
| `SUBSCRIPTION_CREATE` | `SubscriptionCreateEvent` | None | ✅ Supported |
| `SUBSCRIPTION_UPDATE` | `SubscriptionUpdateEvent` | None | ✅ Supported |
| `SUBSCRIPTION_DELETE` | `SubscriptionDeleteEvent` | None | ✅ Supported |

---

## 🌐 REST API Endpoints Coverage

Oxcord provides type-safe HTTP methods in `oxcord-rest` and ergonomic builder structs in `oxcord-builders`.

| Category | Endpoint Scope | Method Coverage | Builder Support |
|---|---|:---:|:---:|
| **Application Commands** | Global & Guild Slash Commands, Permissions, Options | 100% | `CreateCommand`, `EditCommand`, `CreateCommandPermissions` |
| **Interactions** | Responses (Channel Message, Deferred, Modal, Autocomplete, Premium) | 100% | `CreateInteractionResponse`, `CreateInteractionResponseMessage` |
| **Channels** | CRUD, Messages, Pins, Invites, Webhooks, Typing, Permissions | 100% | `CreateMessage`, `EditMessage`, `CreateChannel`, `EditChannel` |
| **Threads** | Start from Message, Start without Message, Join/Leave, Member Management | 100% | `CreateThread`, `EditThread` |
| **Guilds** | CRUD, Channels, Roles, Members, Bans, Pruning, Audit Log, Integrations | 100% | `CreateGuild`, `EditGuild`, `CreateRole`, `EditRole`, `EditMember` |
| **Auto Moderation** | Rule Listing, Creation, Modification, Deletion | 100% | `CreateAutoModRule`, `EditAutoModRule` |
| **Webhooks** | Execute, Edit, Delete, GitHub/Slack Compatible Payloads | 100% | `ExecuteWebhook`, `EditWebhook` |
| **Users** | Current User, Get User, DM Channels, Current Application Info | 100% | `EditCurrentUser` |
| **Polls** | Create Poll, End Poll, Get Voters, Expire Poll | 100% | `CreatePoll` |
| **Monetization** | List Entitlements, Consume Entitlements, Subscriptions | 100% | Native `oxcord-model` structures |

---

## 🎙️ Voice & Audio Infrastructure

Voice protocol implementations in `oxcord-voice` and `oxcord-voice-model`:

| Capability | Specification | Status | Notes |
|---|---|:---:|---|
| **Voice Gateway Protocol** | Voice Gateway v4 and v8 | ✅ Supported | Reconnecting, heartbeats, and speaking states |
| **Audio Transport** | UDP RTP packetization & RTCP | ✅ Supported | Opus 48 kHz stereo framing |
| **XChaCha20-Poly1305** | AEAD cipher (`aead_xchacha20_poly1305_rtpsize`) | ✅ Supported | Modern Discord standard |
| **AES-256-GCM** | AEAD cipher (`aead_aes256_gcm_rtpsize`) | ✅ Supported | High-performance hardware acceleration |
| **XSalsa20-Poly1305** | Legacy ciphers | ❌ Deprecated | Discontinued by Discord; use AEAD modes |
| **DAVE Protocol** | End-to-End Encrypted Voice | ⏳ Roadmap | Architecture ready; awaiting protocol finalization |

---

## 🧩 UI Components & Components V2

| Component | Discord Spec | Status | Builder |
|---|---|:---:|---|
| **Action Rows** | Layout container for buttons and selects | ✅ Supported | `CreateActionRow` |
| **Buttons** | Primary, Secondary, Success, Danger, Link, Premium | ✅ Supported | `CreateButton` |
| **String Select Menu** | Single and multi-value string picker | ✅ Supported | `CreateSelectMenu` |
| **User Select Menu** | Discord user picker | ✅ Supported | `CreateSelectMenu::user` |
| **Role Select Menu** | Server role picker | ✅ Supported | `CreateSelectMenu::role` |
| **Mentionable Select** | User and role picker | ✅ Supported | `CreateSelectMenu::mentionable` |
| **Channel Select** | Guild channel picker with type filters | ✅ Supported | `CreateSelectMenu::channel` |
| **Modals** | Pop-up forms with text inputs | ✅ Supported | `CreateModal`, `CreateInputText` |

---

## 🏗️ Enterprise Scalability Matrix

| Feature | Sub-crate | Solution |
|---|---|---|
| **In-Memory Cache** | `oxcord-cache` | High-concurrency `DashMap` + `parking_lot::RwLock` |
| **Distributed Cache** | `oxcord-cache/redis` | Native `RedisStateBackend` with lazy connection pooling |
| **High-Throughput Proxy** | `oxcord-proxy` | **Hyper 1.0** rate limit bucket coordinator |
| **Distributed Broker** | `oxcord-broker` | In-Process, Redis Pub/Sub, NATS JetStream, Apache Kafka |
| **Session Persistence** | `oxcord-gateway-daemon` | Disk session caching + instant Resume on SIGTERM/SIGINT |
| **Poise-Style Framework**| `oxcord-framework` | Unified slash + prefix command declarations with cooldowns |
