use serenity::model::prelude::*;
#[test]
fn test() {
    let s = include_str!("test.json");
    let v: Guild = serde_json::from_str(s).unwrap();
}
