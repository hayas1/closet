use hyper::StatusCode;
use serde::{Deserialize, Serialize};

pub fn serialize<S>(
    status: &StatusCode,
    anyhow_error: &anyhow::Error,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    (status.as_u16(), anyhow_error.to_string()).serialize(serializer)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<(StatusCode, anyhow::Error), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (status, anyhow_msg) = <(u16, String)>::deserialize(deserializer)?;
    Ok((
        StatusCode::from_u16(status).expect("invalid status code"), // TODO error handling
        anyhow::Error::msg(anyhow_msg),
    ))
}
