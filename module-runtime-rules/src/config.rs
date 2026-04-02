use pay_respects_utils::strings::print_error;
use serde::Deserialize;

use pay_respects_utils::files::config_files;
use pay_respects_utils::{merge, merge_option};

#[derive(Deserialize, Default)]
pub struct ConfigReader {
	pub merge_commands: Option<Vec<Vec<String>>>,
}

pub struct Config {
	pub merge_commands: Option<Vec<Vec<String>>>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			merge_commands: None,
		}
	}
}

impl Config {
	pub fn merge(&mut self, reader: ConfigReader) {
		merge_option!(self, reader, merge_commands);
	}
}

pub fn load_config() -> Config {
	let mut config = Config::default();

	for file in config_files() {
		let content = std::fs::read_to_string(&file).expect("Failed to read config file");
		let reader: ConfigReader = toml::from_str(&content).unwrap_or_else(|_| {
			print_error(&format!(
				"Failed to parse config file at {}. Skipping.",
				file
			));
			ConfigReader::default()
		});
		config.merge(reader);
	}
	config
}

pub fn get_target_rule(executable: &str, config: &Config) -> String {
	if let Some(merge_commands) = &config.merge_commands {
		for merge_command in merge_commands {
			if merge_command.contains(&executable.to_string()) {
				return merge_command[0].clone();
			}
		}
	}
	executable.to_string()
}
