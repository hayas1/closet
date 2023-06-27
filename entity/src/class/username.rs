use serde::{Deserialize, Serialize};

use crate::error::{EntityError, ValidateError};

pub const REGEX: &str = r"^[a-zA-Z0-9_]+$";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Username(String);
impl Username {
    pub fn parse(username: &str) -> Result<Self, EntityError> {
        let re = regex::Regex::new(REGEX).expect("invalid regex");
        if re.is_match(username) {
            Ok(Self(username.into()))
        } else {
            Err(ValidateError::CannotValidateUsername { invalid_username: username.into() })?
        }
    }
}
impl std::fmt::Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::str::FromStr for Username {
    type Err = EntityError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

super::impl_convert_string_value!(Username);
super::impl_into_active_value!(Username);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ok() {
        assert!(Username::parse(r"x").is_ok());
        assert!(Username::parse(r"0").is_ok());
        assert!(Username::parse(r"_").is_ok());
        assert!(Username::parse(r"_________________").is_ok());
        assert!(Username::parse(r"5a").is_ok());
        assert!(Username::parse(r"user1").is_ok());
        assert!(Username::parse(r"CAMEL").is_ok());
        assert!(Username::parse(r"__init__").is_ok());
        assert!(Username::parse(r"_xYW0WYx_").is_ok());
        assert!(Username::parse(r"__MWM1nun").is_ok());
    }

    #[test]
    fn test_validate_err() {
        assert!(Username::parse(r"").is_err());
        assert!(Username::parse(r"xxx@xxx.xxx").is_err());
        assert!(Username::parse(r"con^^").is_err());
        assert!(Username::parse(r"_/\_").is_err());
        assert!(Username::parse(r"$%)'&)'!#$").is_err());
    }
}
