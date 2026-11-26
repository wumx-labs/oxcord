//! Parsing permission bitflags from `"NAME | NAME | ..."` strings, used by
//! `#[oxcord_macros::command(required_permissions = "...", required_bot_permissions = "...")]`
//! codegen (runtime parsing keeps the macro itself simple — this only ever
//! runs once per command, at `Command::builder()` time).

use oxcord_model::Permissions;

/// Parse a `"SEND_MESSAGES | EMBED_LINKS"`-style string into [`Permissions`].
///
/// Names match the `Permissions` associated constants exactly (case-sensitive,
/// same spelling as Discord's permission flag names). Unknown names are
/// silently ignored rather than panicking, since this typically runs from
/// generated code at startup where a typo should show up as "permission
/// check never triggers" rather than crashing the whole bot — write a unit
/// test asserting the parse result if you want a typo caught at CI time.
#[must_use]
pub fn parse_permissions(spec: &str) -> Permissions {
    spec.split('|')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .filter_map(<Permissions as bitflags::Flags>::from_name)
        .fold(Permissions::empty(), |acc, p| acc | p)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_single_and_multiple_flags() {
        assert_eq!(parse_permissions("SEND_MESSAGES"), Permissions::SEND_MESSAGES);
        assert_eq!(
            parse_permissions("SEND_MESSAGES | EMBED_LINKS"),
            Permissions::SEND_MESSAGES | Permissions::EMBED_LINKS
        );
    }

    #[test]
    fn ignores_unknown_names_and_whitespace() {
        assert_eq!(parse_permissions(""), Permissions::empty());
        assert_eq!(parse_permissions("NOT_A_REAL_PERM"), Permissions::empty());
        assert_eq!(parse_permissions("  SEND_MESSAGES  |  EMBED_LINKS  "), parse_permissions("SEND_MESSAGES|EMBED_LINKS"));
    }
}
