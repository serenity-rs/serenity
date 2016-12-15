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

    assert!(user.avatar_url().unwrap().ends_with("/avatars/210/abc.jpg"));

    user.avatar = None;

    assert!(user.avatar_url().is_none());
}
