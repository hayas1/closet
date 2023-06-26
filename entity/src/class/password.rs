use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use serde::{Deserialize, Serialize};

use crate::error::validate::ValidateError;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Password {
    Authenticated(String),
    Unauthenticated,
}
impl Password {
    pub fn is_unauthenticated(&self) -> bool {
        matches!(self, Self::Unauthenticated)
    }
    pub fn hash(raw: &[u8]) -> Result<Self, ValidateError> {
        let salt = SaltString::generate(&mut OsRng);
        let hashed = Argon2::default()
            .hash_password(raw, &salt)
            .map_err(|_| ValidateError::CannotHashPassword)
            .map(|password| password.to_string())?;
        Ok(Self::Authenticated(hashed))
    }
    pub fn verify(&self, raw: &[u8]) -> bool {
        match self {
            Self::Authenticated(hashed) => {
                if let Ok(password) = PasswordHash::new(&hashed) {
                    Argon2::default().verify_password(raw, &password).is_ok()
                } else {
                    false
                }
            }
            Self::Unauthenticated => false,
        }
    }
}
impl Default for Password {
    fn default() -> Self {
        Self::Unauthenticated
    }
}
impl std::fmt::Display for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authenticated(hashed) => write!(f, "{}", hashed),
            Self::Unauthenticated => write!(f, "********"),
        }
    }
}
impl std::str::FromStr for Password {
    // FIXME error handling
    type Err = ValidateError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::Authenticated(
            PasswordHash::new(s).map_err(|_| ValidateError::CannotHashPassword)?.to_string(),
        ))
    }
}

super::impl_convert_string_value!(Password);
super::impl_into_active_value!(Password);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_password_verify() {
        let passwords = vec![
            "password",
            "qLzo92FJY!@dOi3Y1upO",
            "",
            "ku haku space ><><><>< 'quote\" ^^",
            "ሲ䩲飬🎇⍅睷Ậ",
        ];
        for password in passwords {
            let hashed = Password::hash(password.as_bytes()).unwrap();
            assert_ne!(hashed, Password::Authenticated(password.into()));
            assert!(hashed.verify(password.as_bytes()));
        }
    }
}
