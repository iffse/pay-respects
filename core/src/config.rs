use pay_respects_utils::strings::print_error;
use pay_respects_utils::ui::{ColorsReader, UiConfig, UiConfigReader};
use serde::Deserialize;

use pay_respects_utils::files::config_files;
use pay_respects_utils::{merge, merge_option};

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct ConfigReader {
	pub privilege: Option<String>,
	pub merge_commands: Option<Vec<Vec<String>>>,
	pub timeout: Option<u64>,
	pub blocking_commands: Option<Vec<String>>,
	pub eval_method: Option<EvalMethod>,
	pub package_manager: Option<PackageManagerConfig>,
	pub ui: Option<UiConfigReader>,
	pub colors: Option<ColorsReader>,
}

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct PackageManagerConfig {
	pub package_manager: Option<String>,
	pub install_method: Option<InstallMethod>,
}

#[derive(Deserialize, Default, PartialEq)]
pub enum InstallMethod {
	#[default]
	Default,
	System,
	Shell, // only available for nix and guix
}

#[derive(Deserialize, Default, PartialEq)]
pub enum EvalMethod {
	#[default]
	Internal,
	Shell,
}

pub struct Config {
	pub privilege: Option<String>,
	pub merge_commands: Option<Vec<Vec<String>>>,
	pub timeout: u64,
	pub blocking_commands: Option<Vec<String>>,
	pub eval_method: EvalMethod,
	pub package_manager: Option<String>,
	pub install_method: InstallMethod,
	pub ui: UiConfig,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			privilege: None,
			merge_commands: None,
			timeout: 3000,
			blocking_commands: None,
			eval_method: EvalMethod::Internal,
			package_manager: None,
			install_method: InstallMethod::Default,
			ui: UiConfig::default(),
		}
	}
}

impl Config {
	pub fn merge(&mut self, reader: ConfigReader) {
		merge_option!(self, reader, privilege, merge_commands, blocking_commands);
		merge!(self, reader, timeout, eval_method);

		if let Some(reader) = reader.package_manager {
			merge_option!(self, reader, package_manager);
			merge!(self, reader, install_method);
		}

		if let Some(ui) = reader.ui {
			self.ui.merge(ui);
		}
		if let Some(colors) = reader.colors {
			self.ui.colors.merge(colors);
		}
	}

	pub fn set_package_manager(&mut self, package_manager: &str) {
		self.package_manager = Some(package_manager.to_string());
	}

	pub fn set_install_method(&mut self) {
		let package_manager = self.package_manager.as_deref().unwrap();
		if self.install_method == InstallMethod::Default {
			match package_manager {
				"nix" | "guix" => self.install_method = InstallMethod::Shell,
				_ => self.install_method = InstallMethod::System,
			}
		}
	}
}

pub fn load_config() -> Config {
	if std::env::var("_PR_NO_CONFIG").is_ok() {
		return Config::default();
	}

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

#[cfg(test)]
mod tests {
	use super::*;
	use colored::Color;

	#[test]
	fn parses_ui_and_colors() {
		let reader: ConfigReader = toml::from_str(
			r##"
[ui]
prelude = false
hint = false

[colors]
accent = "#ff0000"
normal = "white"
"##,
		)
		.unwrap();

		let mut config = Config::default();
		config.merge(reader);

		assert!(!config.ui.prelude);
		assert!(!config.ui.hint);
		assert_eq!(
			config.ui.colors.accent,
			Color::TrueColor { r: 255, g: 0, b: 0 }
		);
		assert_eq!(config.ui.colors.normal, Some(Color::White));
	}

	#[test]
	fn defaults_keep_the_interface_visible() {
		let reader: ConfigReader = toml::from_str("").unwrap();
		let mut config = Config::default();
		config.merge(reader);

		assert!(config.ui.prelude);
		assert!(config.ui.hint);
		assert_eq!(config.ui.colors.wrong, Color::Red);
		assert_eq!(config.ui.colors.correct, Color::Green);
	}
}
