use std::path::Path;

use pay_respects_utils::files::{config_files, user_config_path};
use pay_respects_utils::merge_option;
use pay_respects_utils::strings::print_error;
use serde::Deserialize;
use toml_edit::{DocumentMut, Item, Table};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
	pub enabled: Option<bool>,
	pub api_key: Option<String>,
	pub api_url: Option<String>,
	pub model: Option<String>,
	pub extra: Option<String>,
	pub extra_body: Option<String>,
	pub locale: Option<String>,
	pub additional_prompt: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
struct ConfigReader {
	module: Option<ModuleConfigReader>,
}

#[derive(Debug, Default, Deserialize)]
struct ModuleConfigReader {
	#[serde(rename = "request-ai")]
	request_ai: Option<Config>,
}

impl Config {
	fn merge(&mut self, reader: Config) {
		merge_option!(
			self,
			reader,
			enabled,
			api_key,
			api_url,
			model,
			extra,
			extra_body,
			locale,
			additional_prompt
		);
	}

	pub fn has_custom_api_key(&self) -> bool {
		let key = match std::env::var("_PR_AI_API_KEY") {
			Ok(key) => Some(key),
			Err(_) => self.api_key.clone(),
		};
		key.is_some_and(|key| !key.is_empty())
	}
}

fn merge_reader(config: &mut Config, reader: ConfigReader) {
	let Some(module) = reader.module else {
		return;
	};
	let Some(request_ai) = module.request_ai else {
		return;
	};
	config.merge(request_ai);
}

fn apply_enabled(content: &str, enabled: bool) -> Result<String, String> {
	let mut document = content
		.parse::<DocumentMut>()
		.map_err(|err| format!("Failed to parse config: {}", err))?;

	let root = document.as_table_mut();
	if !root.contains_key("module") {
		let mut module = Table::new();
		module.set_implicit(true);
		root.insert("module", Item::Table(module));
	}
	let module = root
		.get_mut("module")
		.and_then(Item::as_table_mut)
		.ok_or("`module` is not a table")?;

	match module.get_mut("request-ai") {
		Some(Item::Table(table)) => {
			table.insert("enabled", toml_edit::value(enabled));
		}
		Some(_) => return Err("`module.request-ai` is not a table".to_string()),
		None => {
			let mut table = Table::new();
			table.insert("enabled", toml_edit::value(enabled));
			module.insert("request-ai", Item::Table(table));
		}
	}

	Ok(document.to_string())
}

pub fn set_enabled(enabled: bool) -> Result<(), String> {
	let path = user_config_path();
	let content = match std::fs::read_to_string(&path) {
		Ok(content) => content,
		Err(err) if err.kind() == std::io::ErrorKind::NotFound => String::new(),
		Err(err) => return Err(format!("Failed to read {}: {}", path, err)),
	};

	let content = apply_enabled(&content, enabled).map_err(|err| format!("{}: {}", path, err))?;

	if let Some(parent) = Path::new(&path).parent() {
		std::fs::create_dir_all(parent)
			.map_err(|err| format!("Failed to create {}: {}", parent.display(), err))?;
	}
	std::fs::write(&path, content).map_err(|err| format!("Failed to write {}: {}", path, err))
}

pub fn load() -> Config {
	if std::env::var("_PR_NO_CONFIG").is_ok() {
		return Config::default();
	}

	let mut config = Config::default();

	for file in config_files() {
		let content = match std::fs::read_to_string(&file) {
			Ok(content) => content,
			Err(_) => continue,
		};
		match toml::from_str::<ConfigReader>(&content) {
			Ok(reader) => merge_reader(&mut config, reader),
			Err(_) => print_error(&format!(
				"Failed to parse config file at {}. Skipping.",
				file
			)),
		}
	}

	config
}

#[cfg(test)]
mod tests {
	use super::*;

	fn parse(content: &str) -> Config {
		let reader = toml::from_str::<ConfigReader>(content).unwrap();
		let mut config = Config::default();
		merge_reader(&mut config, reader);
		config
	}

	#[test]
	fn parses_table() {
		let config = parse(
			r#"
[module.request-ai]
enabled = true
api_key = "key"
api_url = "https://example.com/v1/chat/completions"
model = "model"
extra = '"temperature" = 0.5'
extra_body = '{"thinking": false}'
locale = "en-US"
additional_prompt = "be helpful"
"#,
		);

		assert_eq!(config.enabled, Some(true));
		assert_eq!(config.api_key.as_deref(), Some("key"));
		assert_eq!(
			config.api_url.as_deref(),
			Some("https://example.com/v1/chat/completions")
		);
		assert_eq!(config.model.as_deref(), Some("model"));
		assert_eq!(config.extra.as_deref(), Some(r#""temperature" = 0.5"#));
		assert_eq!(config.extra_body.as_deref(), Some(r#"{"thinking": false}"#));
		assert_eq!(config.locale.as_deref(), Some("en-US"));
		assert_eq!(config.additional_prompt.as_deref(), Some("be helpful"));
	}

	#[test]
	fn empty_config_leaves_enabled_unset() {
		let config = parse("");
		assert_eq!(config.enabled, None);
	}

	#[test]
	fn other_sections_are_ignored() {
		let config = parse(
			r#"
search_threshold = 3

[trigram]
minimum_score = 0.5
"#,
		);

		assert_eq!(config.enabled, None);
		assert!(config.api_key.is_none());
	}

	#[test]
	fn saves_enabled_to_empty_config() {
		let content = apply_enabled("", true).unwrap();
		assert!(content.contains("[module.request-ai]"));
		assert!(!content.contains("[module]\n"));
		assert_eq!(parse(&content).enabled, Some(true));
	}

	#[test]
	fn updates_enabled_and_preserves_other_options() {
		let content = apply_enabled(
			r#"
timeout = 3000

[module.request-ai]
enabled = false
api_key = "key"
"#,
			true,
		)
		.unwrap();

		assert!(content.contains("timeout = 3000"));
		let config = parse(&content);
		assert_eq!(config.enabled, Some(true));
		assert_eq!(config.api_key.as_deref(), Some("key"));
	}

	#[test]
	fn saves_disabled_to_config() {
		let content = apply_enabled("", false).unwrap();
		assert_eq!(parse(&content).enabled, Some(false));
	}
}
