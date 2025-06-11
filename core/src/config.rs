use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct Config {
	pub sudo: Option<String>,
	#[serde(default)]
	pub timeout: Timeout,
	#[serde(default)]
	pub package_manager: PackageManagerConfig,
}

#[allow(dead_code)]
#[derive(Deserialize, Default)]
pub struct PackageManagerConfig {
	pub package_manager: Option<String>,
	#[serde(default)]
	pub install_method: InstallMethod,
}

#[derive(Deserialize)]
pub struct Timeout(pub u64);
impl Default for Timeout {
	fn default() -> Self {
		Timeout(3000)
	}
}

#[derive(Deserialize, Default, PartialEq)]
pub enum InstallMethod {
	#[default]
	System,
	// !TODO: Implement other install methods
	// User,
	// Temp,
	Shell,
}

pub fn load_config() -> Config {
	let path = config_path();
	let exists = std::path::Path::new(&path).exists();
	if exists {
		let content = std::fs::read_to_string(&path).expect("Failed to read config file");
		let config: Config = toml::from_str(&content).unwrap_or_else(|_| {
			eprintln!(
				"Failed to parse config file at {}. Using default configuration.",
				path
			);
			Config::default()
		});
		return config;
	}
	Config::default()
}

fn config_path() -> String {
	#[cfg(windows)]
	let xdg_config_home = std::env::var("APPDATA").unwrap();
	#[cfg(not(windows))]
	let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
		.unwrap_or_else(|_| std::env::var("HOME").unwrap() + "/.config");

	format!("{}/pay-respects/config.toml", xdg_config_home)
}
