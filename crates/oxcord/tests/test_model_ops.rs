use std::collections::*;
use std::num::NonZeroU16;
use std::str::FromStr;

use oxcord::model::prelude::*;

fn gen_member() -> Member {
    Member {
        nick: Some("aaaa".to_string()),
        user: User {
            name: "test".into(),
            discriminator: NonZeroU16::new(1432),
            ..User::default()
        },
        ..Default::default()
    }
}

fn gen_guild() -> Guild {
    let m = gen_member();

    Guild {
        members: HashMap::from([(m.user.id, m)]),
        ..Default::default()
    }
}

#[test]
fn member_named_username() {
    let guild = gen_guild();
    let lhs = guild.member_named("test#1432").unwrap().display_name();

    assert_eq!(lhs, gen_member().display_name());
}

#[test]
fn member_named_nickname() {
    let guild = gen_guild();
    let lhs = guild.member_named("aaaa").unwrap().display_name();

    assert_eq!(lhs, gen_member().display_name());
}

#[test]
fn test_user_avatar_and_tag() {
    let mut user = User {
        id: UserId::new(210),
        avatar: Some(ImageHash::from_str("fb211703bcc04ee612c88d494df0272f").unwrap()),
        discriminator: NonZeroU16::new(1432),
        name: "test".to_string(),
        ..Default::default()
    };

    let expected = "/avatars/210/fb211703bcc04ee612c88d494df0272f.webp?size=1024";
    assert!(user.avatar_url().unwrap().ends_with(expected));
    assert!(user.static_avatar_url().unwrap().ends_with(expected));

    user.avatar = Some(ImageHash::from_str("a_fb211703bcc04ee612c88d494df0272f").unwrap());
    let expected = "/avatars/210/a_fb211703bcc04ee612c88d494df0272f.gif?size=1024";
    assert!(user.avatar_url().unwrap().ends_with(expected));
    let expected = "/avatars/210/a_fb211703bcc04ee612c88d494df0272f.webp?size=1024";
    assert!(user.static_avatar_url().unwrap().ends_with(expected));

    user.avatar = None;
    assert!(user.avatar_url().is_none());

    assert_eq!(user.tag(), "test#1432");
}

#[test]
fn default_avatars() {
    let mut user = User {
        discriminator: None,
        id: UserId::new(737323631117598811),
        ..Default::default()
    };

    // New username system
    assert!(user.default_avatar_url().ends_with("5.png"));

    // Legacy username system
    user.discriminator = NonZeroU16::new(1);
    assert!(user.default_avatar_url().ends_with("1.png"));
    user.discriminator = NonZeroU16::new(2);
    assert!(user.default_avatar_url().ends_with("2.png"));
    user.discriminator = NonZeroU16::new(3);
    assert!(user.default_avatar_url().ends_with("3.png"));
    user.discriminator = NonZeroU16::new(4);
    assert!(user.default_avatar_url().ends_with("4.png"));
}
