#![cfg_attr(feature = "cargo-clippy", allow(unreadable_literal))]
#![cfg(all(feature = "builder", feature = "utils"))]

#[macro_use]
extern crate serde_json;
extern crate serenity;

use serde_json::Value;
use serenity::model::channel::{Embed, EmbedField, EmbedFooter, EmbedImage, EmbedVideo};
use serenity::builder::CreateEmbed;
use serenity::utils::{self, Colour};

#[test]
fn test_from_embed() {
    let embed = Embed {
        author: None,
        colour: Colour::new(0xFF0011),
        description: Some("This is a test description".to_string()),
        fields: vec![
            EmbedField {
                inline: false,
                name: "a".to_string(),
                value: "b".to_string(),
            },
            EmbedField {
                inline: true,
                name: "c".to_string(),
                value: "z".to_string(),
            },
        ],
        footer: Some(EmbedFooter {
            icon_url: Some("https://i.imgur.com/XfWpfCV.gif".to_string()),
            proxy_icon_url: None,
            text: "This is a hakase footer".to_string(),
        }),
        image: Some(EmbedImage {
            height: 213,
            proxy_url: "a".to_string(),
            url: "https://i.imgur.com/XfWpfCV.gif".to_string(),
            width: 224,
        }),
        kind: "rich".to_string(),
        provider: None,
        thumbnail: None,
        timestamp: None,
        title: Some("hakase".to_string()),
        url: Some("https://i.imgur.com/XfWpfCV.gif".to_string()),
        video: Some(EmbedVideo {
            height: 213,
            url: "https://i.imgur.com/XfWpfCV.mp4".to_string(),
            width: 224,
        }),
    };

    let builder = CreateEmbed::from(embed)
        .colour(0xFF0011)
        .description("This is a hakase description")
        .image("https://i.imgur.com/XfWpfCV.gif")
        .title("still a hakase")
        .url("https://i.imgur.com/XfWpfCV.gif");

    let built = Value::Object(utils::vecmap_to_json_map(builder.0));

    let obj = json!({
        "color": 0xFF0011,
        "description": "This is a hakase description",
        "title": "still a hakase",
        "type": "rich",
        "url": "https://i.imgur.com/XfWpfCV.gif",
        "fields": [
            {
                "inline": false,
                "name": "a",
                "value": "b",
            },
            {
                "inline": true,
                "name": "c",
                "value": "z",
            },
        ],
        "image": {
            "url": "https://i.imgur.com/XfWpfCV.gif",
        },
        "footer": {
            "text": "This is a hakase footer",
            "icon_url": "https://i.imgur.com/XfWpfCV.gif",
        }
    });

    assert_eq!(built, obj);
}
