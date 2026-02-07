use crate::style::print_error;
use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct ConfigReader {
	pub sudo: Option<String>,
	pub timeout: Option<u64>,
	pub eval_method: Option<EvalMethod>,
	pub package_manager: Option<PackageManagerConfig>,
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
	pub sudo: Option<String>,
	pub timeout: u64,
	pub eval_method: EvalMethod,
	pub package_manager: Option<String>,
	pub install_method: InstallMethod,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			sudo: None,
			timeout: 3,
			eval_method: EvalMethod::Internal,
			package_manager: None,
			install_method: InstallMethod::Default,
		}
	}
}

macro_rules! merge {
	($self:ident, $reader:ident, $($field:ident),*) => {
		$(
			if let Some($field) = $reader.$field {
				$self.$field = $field;
			}
		)*
	};
}

macro_rules! merge_option {
	($self:ident, $reader:ident, $($field:ident),*) => {
		$(
			if let Some($field) = $reader.$field {
				$self.$field = Some($field);
			}
		)*
	};
}

impl Config {
	pub fn merge(&mut self, reader: ConfigReader) {
		merge_option!(self, reader, sudo);
		merge!(self, reader, timeout, eval_method);

		if let Some(reader) = reader.package_manager {
			merge_option!(self, reader, package_manager);
			merge!(self, reader, install_method);
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
	let mut config = Config::default();

	for path in config_paths() {
		if std::path::Path::new(&path).exists() {
			let content = std::fs::read_to_string(&path).expect("Failed to read config file");
			let reader: ConfigReader = toml::from_str(&content).unwrap_or_else(|_| {
				print_error(&format!(
					"Failed to parse config file at {}. Skipping.",
					path
				));
				ConfigReader::default()
			});
			config.merge(reader);
		}
	}

	config
}

fn config_paths() -> Vec<String> {
	let mut paths = system_config_path();
	paths.push(user_config_path());

	#[cfg(debug_assertions)]
	eprintln!("Config paths: {:?}", paths);

	paths
}

fn system_config_path() -> Vec<String> {
	#[cfg(windows)]
	let xdg_config_dirs = std::env::var("PROGRAMDATA")
		.unwrap_or_else(|_| "C:\\ProgramData".to_string())
		.split(';')
		.map(|s| format!("{}/pay-respects/config.toml", s))
		.collect::<Vec<String>>();
	#[cfg(not(windows))]
	let xdg_config_dirs = std::env::var("XDG_CONFIG_DIRS")
		.unwrap_or_else(|_| "/etc/xdg".to_string())
		.split(':')
		.map(|s| format!("{}/pay-respects/config.toml", s))
		.collect::<Vec<String>>();

	xdg_config_dirs
}

fn user_config_path() -> String {
	#[cfg(windows)]
	let xdg_config_home = std::env::var("APPDATA").unwrap();
	#[cfg(not(windows))]
	let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
		.unwrap_or_else(|_| std::env::var("HOME").unwrap() + "/.config");

	format!("{}/pay-respects/config.toml", xdg_config_home)
}
