#!/usr/bin/env python3
"""
Generates docs/public/llms.txt and docs/public/llms-full.txt
covering 100% of the guides and the complete 12-crate API reference for AI consumption.
"""

import os
import glob
import re

BASE_DIR = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DOCS_DIR = os.path.join(BASE_DIR, "docs")
PUBLIC_DIR = os.path.join(DOCS_DIR, "public")
os.makedirs(PUBLIC_DIR, exist_ok=True)

GUIDE_FILES = [
    ("What is OXCORD", "docs/guide/what-is-oxcord.md", "Core philosophy, performance characteristics, and multi-tier scaling"),
    ("Getting Started", "docs/guide/getting-started.md", "Installation, setting up your bot application, and writing your first ping bot"),
    ("Architecture & Modular Crates", "docs/guide/architecture.md", "Detailed breakdown of the 14 modular crates and zero-cost abstractions"),
    ("API Coverage Matrix", "docs/guide/coverage.md", "Complete Discord API v10 Gateway, REST, Voice, and Components coverage breakdown"),
    ("Gateway & Sharding", "docs/guide/gateway.md", "WebSocket connection, Gateway intents, reconnect handling, and transparent sharding"),
    ("Cache Engine", "docs/guide/cache.md", "Lock-free in-memory state caching, cache configuration, and automated sweepers"),
    ("Voice & Audio Driver", "docs/guide/voice.md", "Voice connection protocol, UDP RTP audio pipeline, and AES-GCM encryption"),
    ("Distributed Message Broker", "docs/guide/broker.md", "Pluggable broker abstraction with NATS, Redis Pub/Sub, Kafka, and In-Process bus for large-scale bots"),
    ("Slash Commands & Interactions", "docs/guide/slash-commands.md", "Application commands, subcommands, autocomplete, and interaction responses"),
    ("Webhooks", "docs/guide/webhooks.md", "Creating, executing, editing, and managing Discord Webhooks with builders"),
    ("Migrating from Serenity", "docs/guide/migrating-from-serenity.md", "Step-by-step mapping of types, client initialization, and handler patterns from Serenity to OXCORD"),
    ("Migrating from Twilight", "docs/guide/migrating-from-twilight.md", "Mapping from Twilight's raw model/http/gateway crates to OXCORD's unified ergonomics"),
    ("Example: Basic Ping Bot", "docs/guide/examples/e01-basic-ping.md", "Complete source code and explanation for a minimalist ping bot"),
    ("Example: Distributed Cache Bot", "docs/guide/examples/e22-distributed-cache.md", "Architecture and code for enterprise multi-process bots with shared distributed cache"),
]

CRATE_ORDER = [
    ("oxcord", "Unified facade crate providing high-level ergonomics, client, framework, and full re-exports"),
    ("oxcord-model", "Complete Discord API data models, snowflakes, events, channels, messages, and permissions"),
    ("oxcord-builders", "Type-safe fluent request builders for messages, embeds, buttons, selects, modals, and commands"),
    ("oxcord-rest", "High-performance HTTP client with automated rate-limiting buckets and multipart support"),
    ("oxcord-gateway", "WebSocket connection manager, heartbeat monitor, compression, and shard manager"),
    ("oxcord-cache", "Zero-lock-contention in-memory cache engine for guilds, channels, roles, and members"),
    ("oxcord-voice", "Voice gateway driver, RTP audio streaming, Opus encoder integration, and encryption"),
    ("oxcord-voice-model", "Voice gateway v4/v8 data protocol models and encryption payload structures"),
    ("oxcord-broker", "Distributed microservices message broker supporting In-Process, Redis Pub/Sub, Kafka, and NATS"),
    ("oxcord-framework", "High-level poise-equivalent unified slash + prefix command framework with cooldowns and autocomplete"),
    ("oxcord-macros", "Procedural macros (#[command]) for oxcord-framework declarative commands"),
    ("oxcord-command-attr", "Procedural attribute macros (`#[command]`, `#[group]`, `#[check]`) for legacy text command routing"),
    ("oxcord-proxy", "Standalone reverse-proxy binary for centralized REST API Discord rate-limiting"),
    ("oxcord-gateway-daemon", "Standalone daemon binary for distributed multi-server gateway sharder nodes"),
]

def clean_markdown(content: str) -> str:
    # remove frontmatter
    content = re.sub(r"^---\n.*?\n---\n", "", content, flags=re.DOTALL)
    # remove HTML comments
    content = re.sub(r"<!--.*?-->\n?", "", content)
    # collapse 3+ consecutive newlines
    content = re.sub(r"\n{3,}", "\n\n", content)
    return content.strip()

def build_llms_txt() -> str:
    lines = [
        "# OXCORD: High-Performance Modular Discord Library for Rust (Edition 2024)",
        "",
        "> Official Documentation & AI Developer Context for OXCORD (`oxcord`)",
        "> Repository: https://github.com/wumx-labs/oxcord",
        "> Documentation Portal: https://oxcord.noxlydev.xyz",
        "> Discord Support: https://discord.gg/JnTRems5CT",
        "> Crates.io: https://crates.io/crates/oxcord",
        "",
        "## Overview",
        "OXCORD is an enterprise-grade, ultra-scalable Rust library for the Discord API, built on Rust Edition 2024 and Rust 1.85+.",
        "It scales seamlessly from a single-process weekend bot to massive enterprise bots serving tens of thousands of guilds across distributed container clusters.",
        "",
        "## Core Philosophy & Architecture",
        "- **Zero Lock Contention**: State cache built on concurrent lock-free data structures.",
        "- **Modular Architecture**: 12 independent crates (use only what you need, from pure models to distributed brokers).",
        "- **Distributed by Design**: Native pluggable message brokers (`oxcord-broker`) supporting In-Process, NATS, Kafka, and Redis Pub/Sub.",
        "- **Full Discord Feature Parity**: Components v2, Application Commands, Voice v4/v8, Polls, AutoMod, Onboarding, and Webhooks.",
        "",
        "## Quick Start Example",
        "```rust",
        "use oxcord::prelude::*;",
        "",
        "struct Handler;",
        "",
        "#[async_trait]",
        "impl EventHandler for Handler {",
        "    async fn message(&self, ctx: Context, msg: Message) {",
        "        if msg.content == \"!ping\" {",
        "            if let Err(why) = msg.channel_id.say(&ctx.http, \"Pong! 🏓\").await {",
        "                eprintln!(\"Error sending message: {why:?}\");",
        "            }",
        "        }",
        "    }",
        "}",
        "",
        "#[tokio::main]",
        "async fn main() {",
        "    let token = std::env::var(\"DISCORD_TOKEN\").expect(\"DISCORD_TOKEN must be set\");",
        "    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;",
        "",
        "    let mut client = Client::builder(&token, intents)",
        "        .event_handler(Handler)",
        "        .await",
        "        .expect(\"Error creating client\");",
        "",
        "    if let Err(why) = client.start().await {",
        "        eprintln!(\"Client error: {why:?}\");",
        "    }",
        "}",
        "```",
        "",
        "## Documentation Index",
        "Full detailed text for all sections below is compiled in [llms-full.txt](https://oxcord.noxlydev.xyz/llms-full.txt).",
        ""
    ]

    for title, path, desc in GUIDE_FILES:
        url = "https://oxcord.noxlydev.xyz/" + path.replace("docs/", "").replace(".md", "")
        lines.append(f"- [{title}]({url}): {desc}")

    lines.append("")
    lines.append("## Crates & API Reference")
    for crate_name, desc in CRATE_ORDER:
        url = f"https://docs.rs/{crate_name}"
        lines.append(f"- [`{crate_name}`]({url}): {desc}")

    lines.append("")
    lines.append("## AI Usage Instructions")
    lines.append("For LLM agents (Claude, ChatGPT, Cursor, Copilot) requiring 100% of all guides, architecture deep-dives, and comprehensive API definitions:")
    lines.append("Fetch the complete uncompressed documentation bundle directly at:")
    lines.append("`https://oxcord.noxlydev.xyz/llms-full.txt`")

    return "\n".join(lines) + "\n"

def build_llms_full_txt() -> str:
    sections = [
        "# OXCORD: The Complete Documentation & API Reference for AI",
        "",
        "> Comprehensive guide, architecture manual, and API reference for the OXCORD Discord ecosystem.",
        "> Generated for LLMs, AI agents, Claude, Cursor, and automated toolchains.",
        "> Website: https://oxcord.noxlydev.xyz",
        "> GitHub: https://github.com/wumx-labs/oxcord",
        "> Author: NoxlyDev",
        "> License: Apache-2.0",
        "",
        "---",
        "",
        "# TABLE OF CONTENTS",
        "1. GUIDES & ARCHITECTURE",
        "   - What is OXCORD",
        "   - Getting Started & Quickstart",
        "   - 12-Crate Modular Architecture",
        "   - Gateway & Transparent Sharding",
        "   - Lock-Free Cache Engine",
        "   - Voice & Real-time Audio Driver",
        "   - Distributed Message Broker (NATS / Redis Pub/Sub / Kafka / InProcess)",
        "   - Slash Commands & Component Interactions",
        "   - Discord Webhooks",
        "   - Migrating from Serenity",
        "   - Migrating from Twilight",
        "   - Full End-to-End Code Examples",
        "2. COMPREHENSIVE CRATE API REFERENCES",
        "   - oxcord (Facade & Ergonomics)",
        "   - oxcord-model (Types, Snowflakes, Payloads, Permissions)",
        "   - oxcord-builders (Fluent Message, Embed, Component & Command Builders)",
        "   - oxcord-rest (HTTP Client & Rate Limiting Engine)",
        "   - oxcord-gateway (WebSocket, Shards, Reconnect, Heartbeat)",
        "   - oxcord-cache (Concurrent In-Memory Cache)",
        "   - oxcord-voice & oxcord-voice-model (Audio Driver & Protocol)",
        "   - oxcord-broker (Distributed Microservices Bus & RPC)",
        "   - oxcord-command-attr (Macro Attributes)",
        "   - oxcord-proxy & oxcord-gateway-daemon (Standalone Services)",
        "",
        "---",
        ""
    ]

    # Part 1: Guides
    sections.append("# PART 1: COMPREHENSIVE GUIDES\n")
    for title, path, _ in GUIDE_FILES:
        full_path = os.path.join(BASE_DIR, path)
        if os.path.exists(full_path):
            with open(full_path, "r", encoding="utf-8") as gf:
                content = clean_markdown(gf.read())
            sections.append(f"\n## {title}\n")
            sections.append(content)
            sections.append("\n---")

    # Part 2: Crates API Reference
    sections.append("\n\n# PART 2: CRATES & DETAILED API SPECIFICATION\n")
    for crate_name, desc in CRATE_ORDER:
        crate_readme_path = os.path.join(BASE_DIR, "crates", crate_name, "README.md")
        crate_cargo_path = os.path.join(BASE_DIR, "crates", crate_name, "Cargo.toml")
        crate_lib_path = os.path.join(BASE_DIR, "crates", crate_name, "src", "lib.rs")
        crate_main_path = os.path.join(BASE_DIR, "crates", crate_name, "src", "main.rs")

        sections.append(f"\n## Crate: `{crate_name}`")
        sections.append(f"**Description**: {desc}")
        sections.append(f"**Docs.rs**: https://docs.rs/{crate_name}")
        sections.append(f"**Crates.io**: https://crates.io/crates/{crate_name}\n")

        if os.path.exists(crate_cargo_path):
            with open(crate_cargo_path, "r", encoding="utf-8") as cf:
                cargo_text = cf.read().strip()
            sections.append("### Manifest (`Cargo.toml`)")
            sections.append("```toml")
            sections.append(cargo_text)
            sections.append("```\n")

        if os.path.exists(crate_readme_path):
            with open(crate_readme_path, "r", encoding="utf-8") as rf:
                readme_text = clean_markdown(rf.read())
            # strip title if repeated
            readme_text = re.sub(r"^#\s+" + re.escape(crate_name) + r".*?\n", "", readme_text, flags=re.IGNORECASE)
            sections.append("### Crate Overview & Features")
            sections.append(readme_text)
            sections.append("")

        source_file = None
        source_label = None
        if os.path.exists(crate_lib_path):
            source_file = crate_lib_path
            source_label = "src/lib.rs"
        elif os.path.exists(crate_main_path):
            source_file = crate_main_path
            source_label = "src/main.rs"

        if source_file:
            with open(source_file, "r", encoding="utf-8") as sf:
                code_text = sf.read().strip()
            # extract module doc comment and public exports
            module_doc = []
            pub_exports = []
            for line in code_text.splitlines():
                if line.startswith("//!"):
                    module_doc.append(line.replace("//!", "").strip())
                elif line.startswith("pub ") or line.startswith("#[macro_export]"):
                    pub_exports.append(line)

            if module_doc:
                sections.append(f"### Module Documentation (`{source_label}`)")
                sections.append("\n".join(module_doc[:60]))
                sections.append("")

            if pub_exports:
                sections.append("### Primary Public Exports")
                sections.append("```rust")
                sections.append("\n".join(pub_exports[:80]))
                sections.append("```")

        sections.append("\n---")

    return "\n".join(sections) + "\n"

if __name__ == "__main__":
    llms_txt = build_llms_txt()
    llms_txt_path = os.path.join(PUBLIC_DIR, "llms.txt")
    with open(llms_txt_path, "w", encoding="utf-8") as f:
        f.write(llms_txt)
    print(f"Generated {llms_txt_path} ({len(llms_txt)} bytes)")

    llms_full_txt = build_llms_full_txt()
    llms_full_path = os.path.join(PUBLIC_DIR, "llms-full.txt")
    with open(llms_full_path, "w", encoding="utf-8") as f:
        f.write(llms_full_txt)
    print(f"Generated {llms_full_path} ({len(llms_full_txt)} bytes)")
