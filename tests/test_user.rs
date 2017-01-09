extern crate serenity;

use serenity::model::{User, UserId};

fn gen() -> User {
    User {
        id: UserId(210),
        avatar: Some("abc".to_owned()),
        bot: true,
        discriminator: "1432".to_owned(),
        name: "test".to_owned(),
    }
}

#[test]
fn test_core() {
    let mut user = gen();

    assert!(user.avatar_url().unwrap().ends_with("/avatars/210/abc.webp?size=1024"));
    assert!(user.static_avatar_url().unwrap().ends_with("/avatars/210/abc.webp?size=1024"));

    user.avatar = Some("a_aaa".to_owned());
    assert!(user.avatar_url().unwrap().ends_with("/avatars/210/a_aaa.gif?size=1024"));
    assert!(user.static_avatar_url().unwrap().ends_with("/avatars/210/a_aaa.webp?size=1024"));

    user.avatar = None;
    assert!(user.avatar_url().is_none());
}

#[test]
fn default_avatars() {
    let mut user = gen();

    user.discriminator = "0".to_owned();
    assert!(user.default_avatar_url().unwrap().ends_with("0.png"));
    user.discriminator = "1".to_owned();
    assert!(user.default_avatar_url().unwrap().ends_with("1.png"));
    user.discriminator = "2".to_owned();
    assert!(user.default_avatar_url().unwrap().ends_with("2.png"));
    user.discriminator = "3".to_owned();
    assert!(user.default_avatar_url().unwrap().ends_with("3.png"));
    user.discriminator = "4".to_owned();
    assert!(user.default_avatar_url().unwrap().ends_with("4.png"));
}
