use serde::de::{Deserialize, Deserializer, Error as _};

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
    loop_errors(&map, &mut errors, &mut path).map_err(D::Error::custom)?;

    Ok(errors)
}

fn make_error(
    errors_value: &Value,
    errors: &mut Vec<DiscordJsonSingleError>,
    path: &[&str],
) -> StdResult<(), &'static str> {
    let found_errors = errors_value.as_array().ok_or("expected array")?;

    for error in found_errors {
        let error_object = error.as_object().ok_or("expected object")?;

        errors.push(DiscordJsonSingleError {
            code: error_object
                .get("code")
                .ok_or("expected code")?
                .as_str()
                .ok_or("expected string")?
                .to_owned(),
            message: error_object
                .get("message")
                .ok_or("expected message")?
                .as_str()
                .ok_or("expected string")?
                .to_owned(),
            path: path.join("."),
        });
    }
    Ok(())
}

fn loop_errors<'a>(
    value: &'a Value,
    errors: &mut Vec<DiscordJsonSingleError>,
    path: &mut Vec<&'a str>,
) -> StdResult<(), &'static str> {
    for (key, value) in value.as_object().ok_or("expected object")? {
        if key == "_errors" {
            make_error(value, errors, path)?;
        } else {
            path.push(key);
            loop_errors(value, errors, path)?;
            path.pop();
        }
    }
    Ok(())
}
