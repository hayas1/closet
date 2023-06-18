use serde::{Deserialize, Serialize};

pub fn serialize<S>(anyhow_error: &anyhow::Error, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    anyhow_error.to_string().serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<anyhow::Error, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let anyhow_msg = String::deserialize(deserializer)?;
    Ok(anyhow::Error::msg(anyhow_msg))
}
