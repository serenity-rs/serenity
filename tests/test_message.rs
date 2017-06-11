extern crate serde;
extern crate serde_json;
extern crate serenity;

use serde::de::Deserialize;
use serde_json::Value;
use serenity::model::Message;
use std::fs::File;

macro_rules! p {
    ($s:ident, $filename:expr) => ({
        let f = File::open(concat!("./tests/resources/", $filename, ".json")).unwrap();
        let v = serde_json::from_reader::<File, Value>(f).unwrap();

        $s::deserialize(v).unwrap()
    })
}

#[test]
fn test_footer_deser() {
    let mut message = p!(Message, "message_footer_1");

    assert_eq!(message.embeds.remove(0).footer.unwrap().text, "2005-09-26 - 2013-09-26");
}
