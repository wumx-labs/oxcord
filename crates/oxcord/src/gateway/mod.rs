//! The gateway module contains the pieces responsible for maintaining a
//! WebSocket connection with Discord.

mod bridge;

pub use oxcord_gateway::*;

pub use self::bridge::*;
