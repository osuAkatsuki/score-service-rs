use serde::de::{Error, Unexpected};
use serde::{Deserialize, Deserializer};

pub fn format_bool(value: bool) -> &'static str {
    match value {
        true => "True",
        false => "False",
    }
}

/// The osu! client sends booleans in query strings as `0` and `1`
pub fn deserialize_bool<'de, D: Deserializer<'de>>(deserializer: D) -> Result<bool, D::Error> {
    let value = String::deserialize(deserializer)?;
    match value.as_str() {
        "1" | "true" | "True" => Ok(true),
        "0" | "false" | "False" => Ok(false),
        other => Err(D::Error::invalid_value(
            Unexpected::Str(other),
            &"a boolean",
        )),
    }
}

pub const DATETIME_FORMAT_STR: &str = "%y%m%d%H%M%S";
