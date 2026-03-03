use serde::Serialize;

use crate::error::Result;

pub fn to_pretty_json<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string_pretty(value).map_err(Into::into)
}
