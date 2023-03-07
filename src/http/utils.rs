use serde::de::{Deserialize, Deserializer};

use crate::http::error::DiscordJsonSingleError;
use crate::internal::prelude::*;

#[allow(clippy::missing_errors_doc)]
pub fn deserialize_errors<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<Vec<DiscordJsonSingleError>, D::Error> {
    let map: Value = Value::deserialize(deserializer)?;

    if !map.is_object() {
        return Ok(vec![]);
    }

    let mut errors = Vec::new();

    loop_errors(&map, &mut errors, &[]);

    Ok(errors)
}

fn loop_errors(value: &Value, errors: &mut Vec<DiscordJsonSingleError>, path: &[String]) {
    for (key, looped) in value.as_object().expect("expected object").iter() {
        let object = looped.as_object().expect("expected object");
        if object.contains_key("_errors") {
            let found_errors = object
                .get("_errors")
                .expect("expected _errors")
                .as_array()
                .expect("expected array")
                .clone();

            for error in found_errors {
                let error_object = error.as_object().expect("expected object");
                let mut object_path = path.to_owned();

                object_path.push(key.to_string());

                errors.push(DiscordJsonSingleError {
                    code: error_object
                        .get("code")
                        .expect("expected code")
                        .as_str()
                        .expect("expected string")
                        .to_owned(),
                    message: error_object
                        .get("message")
                        .expect("expected message")
                        .as_str()
                        .expect("expected string")
                        .to_owned(),
                    path: object_path.join("."),
                });
            }
            continue;
        }

        let mut new_path = path.to_owned();
        new_path.push(key.to_string());

        loop_errors(looped, errors, &new_path);
    }
}
