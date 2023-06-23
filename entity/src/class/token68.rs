use serde::{Deserialize, Serialize};

pub const REGEX: &str = r"^[a-zA-Z0-9_./~+-]+=*$";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Token(String);

impl Token {
    pub fn parse(token: &str) -> Result<Self, crate::error::validate::ValidateError> {
        let re = regex::Regex::new(REGEX).expect("invalid regex");
        if re.is_match(token) {
            Ok(Self(token.into()))
        } else {
            Err(crate::error::validate::ValidateError::CannotValidateToken68 {
                invalid_token: token.into(),
            })
        }
    }
}
impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::str::FromStr for Token {
    type Err = crate::error::validate::ValidateError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

super::impl_convert_string_value!(Token);
super::impl_into_active_value!(Token);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ok() {
        assert!(Token::parse(r"13qv0nu0pw3u90mjf92jm_Bq2j4-59").is_ok());
        assert!(Token::parse(r"-._~+/").is_ok());
        assert!(Token::parse(r"__========").is_ok());
        assert!(Token::parse(r"ascii").is_ok());
        assert!(Token::parse(r"jjq9wro4jq243po43=====================================").is_ok());
        assert!(Token::parse(r"jjq9wro4jq243po43=").is_ok());
        assert!(Token::parse(r"jjq9wro4jq243po43").is_ok());
    }

    #[test]
    fn test_validate_err() {
        assert!(Token::parse(r"13qv0==nu0pw3u90mjf92jm_Bq2j4-59").is_err());
        assert!(Token::parse(r"-._~+/?").is_err());
        assert!(Token::parse(r"_\_========").is_err());
        assert!(Token::parse(r"dd09*234vf===").is_err());
        assert!(Token::parse(r"==asf-lkj=====================================").is_err());
        assert!(Token::parse(r"kkk=jj88122").is_err());
        assert!(Token::parse(r"is_err!=").is_err());
    }
}
