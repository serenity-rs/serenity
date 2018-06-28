#![cfg(feature = "http-client")]

extern crate serenity;

use serenity::http::AttachmentType;
use std::path::Path;

#[test]
fn test_attachment_type() {
    assert!(match AttachmentType::from(Path::new("./dogs/corgis/kona.png")) {
        AttachmentType::Path(_) => true,
        _ => false,
    });
    assert!(match AttachmentType::from("./cats/copycat.png") {
        AttachmentType::Path(_) => true,
        _ => false,
    });
}
