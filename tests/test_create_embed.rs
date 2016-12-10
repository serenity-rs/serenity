extern crate serde_json;
extern crate serenity;

use serde_json::builder::ObjectBuilder;
use serde_json::Value;
use serenity::model::{Embed, EmbedField, EmbedImage};
use serenity::utils::builder::CreateEmbed;
use serenity::utils::Colour;

#[test]
fn from_embed() {
    let embed = Embed {
        author: None,
        colour: Colour::new(0xFF0011),
        description: Some("This is a test description".to_owned()),
        fields: Some(vec![
            EmbedField {
                inline: false,
                name: "a".to_owned(),
                value: "b".to_owned(),
            },
            EmbedField {
                inline: true,
                name: "c".to_owned(),
                value: "z".to_owned(),
            },
        ]),
        image: Some(EmbedImage {
            height: 213,
            proxy_url: "a".to_owned(),
            url: "https://i.imgur.com/q9MqLqZ.png".to_owned(),
            width: 224,
        }),
        kind: "rich".to_owned(),
        provider: None,
        thumbnail: None,
        timestamp: None,
        title: Some("funny cat meme".to_owned()),
        url: Some("https://i.imgur.com/q9MqLqZ.png".to_owned()),
        video: None,
    };

    let builder = CreateEmbed::from(embed)
        .colour(0xFF0000)
        .description("This is a cat description")
        .title("still a funny cat meme")
        .url("https://i.imgur.com/q9MqLqZ.jpg")
        .image(|i| i.url("https://i.imgur.com/q9MqLqZ.jpg"));

    let built = Value::Object(builder.0);

    let obj = ObjectBuilder::new()
        .insert("color", 0xFF0000)
        .insert("description", "This is a cat description")
        .insert_array("fields", |a| a
            .push_object(|o| o
                .insert("inline", false)
                .insert("name", "a")
                .insert("value", "b"))
            .push_object(|o| o
                .insert("inline", true)
                .insert("name", "c")
                .insert("value", "z")))
        .insert_object("image", |o| o
            .insert("url", "https://i.imgur.com/q9MqLqZ.jpg"))
        .insert("title", "still a funny cat meme")
        .insert("type", "rich")
        .insert("url", "https://i.imgur.com/q9MqLqZ.jpg")
        .build();

    assert_eq!(built, obj);
}
