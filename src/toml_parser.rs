use std::collections::HashMap;
use crate::error::{Result, Error::TOMLStructureError};

pub fn parse_toml(toml: &str) -> Result<HashMap<String, String>> {
    let toml_value = toml.parse::<toml::Value>()?;

    if let toml::Value::Table(table) = toml_value {
        table
            .into_iter()
            .map(|(key, value)| {
                if let toml::Value::String(string) = value {
                    Ok((key, string))
                } else {
                    Err(TOMLStructureError)
                }
            })
            .collect()
    } else {
        Err(TOMLStructureError)
    }
}
