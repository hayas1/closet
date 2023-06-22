use serde::{Deserialize, Serialize};

pub const REGEX: &str = r"^[a-zA-Z0-9_.,!?@$%&#/^~+=-]+$";
#[derive(Deserialize)]
#[serde(try_from = "String")]
pub struct RawPassword(String);
impl RawPassword {
    pub fn new(rpw: String) -> Result<Self, crate::error::validate::ValidateError> {
        let re = regex::Regex::new(REGEX).expect("invalid regex");
        if re.is_match(&rpw) {
            Ok(Self(rpw))
        } else {
            Err(crate::error::validate::ValidateError::ContainUnavailableCharacterPassword)
        }
    }

    pub fn hashed(self) -> Result<HashedPassword, crate::error::validate::ValidateError> {
        pwhash::bcrypt::hash(self.0)
            .expect("cannot use rng") // TODO do not use `expect`, should make error struct and set 5xx status_code
            .try_into()
    }
}
impl TryFrom<String> for RawPassword {
    type Error = crate::error::validate::ValidateError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::new(s)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct HashedPassword(String);
impl HashedPassword {
    pub fn verify_password(password: RawPassword, hashed: &HashedPassword) -> bool {
        pwhash::bcrypt::verify(password.0, &hashed.0)
    }
    pub fn verify(&self, password: RawPassword) -> bool {
        Self::verify_password(password, self)
    }
    pub fn same(&self, hashed: &HashedPassword) -> bool {
        self.0 == hashed.0
    }
}
impl std::fmt::Display for HashedPassword {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::str::FromStr for HashedPassword {
    type Err = crate::error::validate::ValidateError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.to_string()))
    }
}
impl TryFrom<RawPassword> for HashedPassword {
    type Error = crate::error::validate::ValidateError;
    fn try_from(rpw: RawPassword) -> Result<Self, Self::Error> {
        rpw.hashed()
    }
}

super::impl_convert_string_value!(HashedPassword);
super::impl_into_active_value!(HashedPassword);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_password_hash() {
        let password = "password".to_string();
        let raw = RawPassword::try_from(password.clone()).unwrap();
        let raw2 = RawPassword::try_from(password).unwrap();
        let hashed = HashedPassword::try_from(raw2).unwrap();
        assert_eq!(raw.0, "password");
        assert_ne!(hashed.0, "password");
        assert!(hashed.verify(raw));
        // assert!(HashedPassword::verify_password(raw, &hashed)); // raw is moved
    }

    #[test]
    fn test_validate_ok() {
        assert!(RawPassword::try_from("password".to_string()).is_ok());
        assert!(RawPassword::try_from("con^^".to_string()).is_ok());
        assert!(RawPassword::try_from("qLzo92FJY!@dOi3Y1upO".to_string()).is_ok());
    }

    #[test]
    fn test_validate_err() {
        assert!(RawPassword::try_from("".to_string()).is_err());
        assert!(RawPassword::try_from("><><><><".to_string()).is_err());
        assert!(RawPassword::try_from("ku haku space".to_string()).is_err());
        assert!(RawPassword::try_from("'quote\"".to_string()).is_err());
    }
}
