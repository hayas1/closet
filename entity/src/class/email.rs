use serde::{Deserialize, Serialize};

pub const LOCAL_PART_REGEX: &str = r"[a-zA-Z0-9_+-]+(\.[a-zA-Z0-9_+-]+)*";
pub const DOMAIN_REGEX: &str = r"([a-zA-Z0-9][a-zA-Z0-9-]*\.)+[a-zA-Z]{2,}";

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Email {
    local_part: String,
    domain: String,
}

impl Email {
    pub fn parse(email: &str) -> Result<Self, crate::error::validate::ValidateError> {
        let regexp = format!("^{}@{}$", LOCAL_PART_REGEX, DOMAIN_REGEX);
        let re = regex::Regex::new(&regexp).expect("invalid regex");
        if re.is_match(email) {
            let splitted = email.split('@').collect::<Vec<_>>();
            let (local_part, domain) = (splitted[0].to_string(), splitted[1].to_string());
            Ok(Self { local_part, domain })
        } else {
            Err(crate::error::validate::ValidateError::CannotValidateEmail {
                invalid_email: email.into(),
            })
        }
    }
}
impl std::fmt::Display for Email {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.local_part, self.domain)
    }
}
impl std::str::FromStr for Email {
    type Err = crate::error::validate::ValidateError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

super::impl_convert_string_value!(Email);
super::impl_into_active_value!(Email);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_ok() {
        assert!(Email::parse(r"all-people.many@company.com").is_ok());
        assert!(Email::parse(r"xxx@xxx.xxx").is_ok());
        assert!(Email::parse(r"+-.++--@sign.only").is_ok());
        assert!(Email::parse(r"x.+0@e.co").is_ok());
        assert!(Email::parse(r"s@s.ss").is_ok());
        assert!(Email::parse(r"dot.in@doma.in").is_ok());
        assert!(Email::parse(r"hyphen-in@domain-is.allowed").is_ok());
    }

    #[test]
    fn test_validate_err() {
        assert!(Email::parse(r"").is_err());
        assert!(Email::parse(r"@").is_err());
        assert!(Email::parse(r"a@").is_err());
        assert!(Email::parse(r"@xxx.xxx").is_err());
        assert!(Email::parse(r"no_at").is_err());
        assert!(Email::parse(r".@start-with.dot").is_err());
        assert!(Email::parse(r".start@with.dot").is_err());
        assert!(Email::parse(r"one-char@domain.en.d").is_err());
        assert!(Email::parse(r"dot.prev.@at.mark").is_err());
        assert!(Email::parse(r"no.dot.in@domain").is_err());
        assert!(Email::parse(r"double..is@in.valid").is_err());
        assert!(Email::parse(r"double@at@mark-is.in.valid").is_err());
        assert!(Email::parse(r"=)('%=!)#($'=)($=#)(%'=").is_err());
    }
}
