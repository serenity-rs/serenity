#![cfg(feature = "utils")]

extern crate serenity;

use serenity::utils::*;

#[test]
fn test_is_nsfw() {
    assert!(!is_nsfw("general"));
    assert!(is_nsfw("nsfw"));
    assert!(is_nsfw("nsfw-test"));
    assert!(!is_nsfw("nsfw-"));
    assert!(!is_nsfw("général"));
    assert!(is_nsfw("nsfw-général"));
}
