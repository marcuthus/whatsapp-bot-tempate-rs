use std::env;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("a variável de ambiente `{0}` não foi definida")]
    Missing(&'static str),
}

#[derive(Debug, Clone)]
pub struct Config {
    pub admin_number: String,
    pub pairing_number: Option<String>,
    pub prefix: char,
    pub database_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        Ok(Self {
            admin_number: required("WHATSAPP_ADMIN_NUMBER")?,
            pairing_number: optional("WHATSAPP_PAIRING_NUMBER"),
            prefix: '!',
            database_path: String::from("auth/whatsapp.db"),
        })
    }
}

fn required(key: &'static str) -> Result<String, ConfigError> {
    optional(key).ok_or(ConfigError::Missing(key))
}

fn optional(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_fails_when_variable_is_missing() {
        let error = required("WHATSAPP_BOT_RS_UNSET_VARIABLE").unwrap_err();

        assert_eq!(
            error.to_string(),
            "a variável de ambiente `WHATSAPP_BOT_RS_UNSET_VARIABLE` não foi definida"
        );
    }

    #[test]
    fn optional_is_none_when_variable_is_missing() {
        assert_eq!(optional("WHATSAPP_BOT_RS_UNSET_VARIABLE"), None);
    }
}
