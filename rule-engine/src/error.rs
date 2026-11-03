use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum RuleError {
	Io(std::io::Error),
	Toml(toml::de::Error),
	Invalid(String),
}

impl Display for RuleError {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			RuleError::Io(err) => write!(f, "{}", err),
			RuleError::Toml(err) => write!(f, "{}", err),
			RuleError::Invalid(message) => write!(f, "{}", message),
		}
	}
}

impl std::error::Error for RuleError {}

impl From<std::io::Error> for RuleError {
	fn from(err: std::io::Error) -> Self {
		RuleError::Io(err)
	}
}

impl From<toml::de::Error> for RuleError {
	fn from(err: toml::de::Error) -> Self {
		RuleError::Toml(err)
	}
}
