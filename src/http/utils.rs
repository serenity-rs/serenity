use serde::{Deserialize, Deserializer};

use crate::http::error::DiscordJsonSingleError;
use crate::internal::prelude::{StdResult, Value};

pub fn deserialize_errors<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> StdResult<Vec<DiscordJsonSingleError>, D::Error> {
    let map: Value = Value::deserialize(deserializer)?;

    if !map.is_object() {
        return Ok(vec![]);
    }

    let mut errors: Vec<DiscordJsonSingleError> = vec![];

    loop_errors(map, &mut errors, &mut Vec::new());

    Ok(errors)
}

fn loop_errors(value: Value, errors: &mut Vec<DiscordJsonSingleError>, path: &mut Vec<String>) {
    for (key, looped) in value.as_object().unwrap().iter() {
        let object = looped.as_object().unwrap();
        if object.contains_key("_errors") {
            let found_errors = object.get("_errors").unwrap().as_array().unwrap().to_owned();
            for error in found_errors {
                let real_object = error.as_object().unwrap();
                let mut object_path = path.clone();

                object_path.push(key.to_string());

                errors.push(DiscordJsonSingleError {
                    code: real_object.get("code").unwrap().as_str().unwrap().to_owned(),
                    message: real_object.get("message").unwrap().as_str().unwrap().to_owned(),
                    path: object_path.join("."),
                });
            }
            continue;
        }
        path.push(key.to_string());

        loop_errors(looped.clone(), errors, path);
    }
}
