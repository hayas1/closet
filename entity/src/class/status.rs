use serde::{Deserialize, Serialize};

use crate::error::{EntityError, ValidateError};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    Ok,
    Ng,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::json!(self).as_str().ok_or_else(|| std::fmt::Error)?)
    }
}
impl std::str::FromStr for Status {
    type Err = EntityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_value(serde_json::json!(s))
            .map_err(|_| ValidateError::UnrecognizedStatus)?
    }
}

super::impl_convert_string_value!(Status);
super::impl_into_active_value!(Status);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde() {
        let ok = Status::Ok;
        let ser = serde_json::json!(ok);
        assert_eq!(ser.as_str().unwrap(), "ok");

        let ng = serde_json::json!("ng");
        let de = serde_json::from_value::<Status>(ng);
        assert_eq!(de.unwrap(), Status::Ng);
    }
}
