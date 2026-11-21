<p align="center">
  <img src="../assets/banner.png" alt="OXCORD Banner" width="100%">
</p>

# OXCORD Examples

<p align="center">
  <a href="../README.md"><img src="https://img.shields.io/badge/examples-25%20runnable%20bots-brightgreen?logo=rust&logoColor=white" alt="Examples"></a>
  <a href="../LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue.svg?logo=apache&logoColor=white" alt="License"></a>
  <a href="https://discord.com/developers/docs"><img src="https://img.shields.io/badge/Discord%20API-v10-5865F2.svg?logo=discord&logoColor=white" alt="Discord API"></a>
  <a href="https://discord.gg/JnTRems5CT"><img src="https://img.shields.io/badge/join%20us-on%20discord-5865F2?logo=discord&logoColor=white" alt="Discord"></a>
</p>

<p align="center">
  <a href="../README.md"><b>Workspace Root</b></a> •
  <a href="https://oxcord.noxlydev.xyz"><b>Documentation</b></a> •
  <a href="https://discord.gg/JnTRems5CT"><b>Discord Community</b></a>
</p>

Comprehensive collection of 25 runnable examples demonstrating patterns from minimal ping bots to large-scale distributed architectures.

Voice support is natively included in `oxcord-voice` and demonstrated in [`e21_voice_playback`](e21_voice_playback). Distributed broker sharding is demonstrated in [`e23_broker_distributed_bot`](e23_broker_distributed_bot), [`e24_stateless_worker`](e24_stateless_worker), and [`e25_full_enterprise_bot`](e25_full_enterprise_bot).

To run any example, set your bot token:
```bash
export DISCORD_TOKEN="your_bot_token_here"
```

## Running Examples

Run any example directly via Cargo:
```bash
cargo run -p e01_basic_ping_bot
cargo run -p e14_slash_commands
cargo run -p e20_components_v2
cargo run -p e21_voice_playback
cargo run -p e25_full_enterprise_bot
```

    ```
    1 => Basic Ping Bot: A bare minimum oxcord application.
    2 => Transparent Guild Sharding: How to use sharding and shared cache.
    3 => Structure Utilities: Simple usage of the utils feature.
    4 => Message Builder: A demonstration of the message builder utility, to generate messages safely.
    5 => Command Framework: The main example, where it's demonstrated how to use oxcord's command framework,
    along with most of its utilities.
    This example also shows how to share data between events and commands, using `Context.data`
    6 => Simple Bot Structure: An example showing the recommended file structure to use.
    7 => Env Logging: How to use the tracing crate along with oxcord.
    8 => Shard Manager: How to get started with using the shard manager.
    9 => Create Message Builder: How to send embeds and files.
    10 => Collectors: How to use the collectors feature to wait for messages and reactions.
    11 => Gateway Intents: How to use intents to limit the events the bot will receive.
    12 => Global Data: How to use the client data to share data between commands and events safely.
    13 => Parallel Loops: How to run tasks in a loop with context access.
    Additionally, show how to send a message to a specific channel.
    14 => Slash Commands: How to use the low level slash command API.
    15 => Simple Dashboard: A simple dashboard to control and monitor the bot with `rillrate`.
    16 => SQLite Database: How to run an embedded SQLite database alongside the bot using SQLx
    17 => Message Components: How to structure and use buttons and select menus
    18 => Webhook: How to construct and call a webhook
    19 => Interactions Endpoint: How to receive interactions via HTTP webhooks
    20 => Components V2: Modern Discord Components V2 (Containers, Sections, TextDisplay, Separators)
    ```

2. Manually running:

- Clone the repository: `git clone https://github.com/wumx-labs/oxcord.git`
- Run the example of choice, selected using the `-p` flag: `cargo run --release -p e01_basic_ping_bot `

3. Copy Paste:

- Copy the contents of the example into your local binary project\
(created via `cargo new test-project --bin`)\
and ensuring that the contents of the `Cargo.toml` file
contains that of the example's `[dependencies]` section,\
and _then_ executing `cargo run`.

### Questions

If you have any questions, feel free to submit an issue with what can be
clarified.

### Contributing

If you add a new example also add it to the following files:

- `.github/workflows/ci.yml`
- `Makefile.toml`
- `examples/README.md`
