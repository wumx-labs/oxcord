//! A collection of newtypes defining type-strong IDs.
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::util::json_safe_u64;

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct GuildId(#[serde(with = "json_safe_u64")] pub u64);

impl fmt::Display for GuildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<u64> for GuildId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<GuildId> for u64 {
    fn from(id: GuildId) -> Self {
        id.0
    }
}

#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct UserId(#[serde(with = "json_safe_u64")] pub u64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<UserId> for u64 {
    fn from(id: UserId) -> Self {
        id.0
    }
}
