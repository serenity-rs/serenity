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
    let mut path = Vec::new();
    loop_errors(&map, &mut errors, &mut path);

    Ok(errors)
}

fn make_error(errors_value: &Value, errors: &mut Vec<DiscordJsonSingleError>, path: &[&str]) {
    let found_errors = errors_value.as_array().expect("expected array").clone();

    for error in found_errors {
        let error_object = error.as_object().expect("expected object");

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
            path: path.join("."),
        });
    }
}

fn loop_errors<'a>(
    value: &'a Value,
    errors: &mut Vec<DiscordJsonSingleError>,
    path: &mut Vec<&'a str>,
) {
    for (key, value) in value.as_object().expect("expected object") {
        if key == "_errors" {
            make_error(value, errors, path);
        } else {
            path.push(key);
            loop_errors(value, errors, path);
            path.pop();
        }
    }
}
