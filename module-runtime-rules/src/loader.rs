use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use pay_respects_rule_engine::{RuleSet, parse_file};

#[derive(Clone)]
pub struct LoadedRule {
	pub path: PathBuf,
	pub rule: RuleSet,
}

pub struct RuleLoader {
	dirs: Vec<PathBuf>,
	cache: HashMap<String, Option<LoadedRule>>,
	deprecation_warned: HashSet<PathBuf>,
	function_warned: HashSet<PathBuf>,
}

impl RuleLoader {
	pub fn new() -> Self {
		Self {
			dirs: rule_dirs(),
			cache: HashMap::new(),
			deprecation_warned: HashSet::new(),
			function_warned: HashSet::new(),
		}
	}

	pub fn load(&mut self, command: &str) -> Option<&LoadedRule> {
		if !self.cache.contains_key(command) {
			let loaded = load_uncached(&self.dirs, command);
			if let Some(loaded) = &loaded {
				if loaded.rule.version < 2 && self.deprecation_warned.insert(loaded.path.clone()) {
					eprintln!(
						"runtime-rules: warning: {} uses the deprecated rule format; see rules.md#version-2",
						loaded.path.display()
					);
				}
				if loaded.rule.has_functions() && self.function_warned.insert(loaded.path.clone()) {
					eprintln!(
						"runtime-rules: warning: {} contains compile-time functions, which are skipped",
						loaded.path.display()
					);
				}
			}
			self.cache.insert(command.to_string(), loaded);
		}

		self.cache.get(command).and_then(Option::as_ref)
	}
}

fn load_uncached(dirs: &[PathBuf], command: &str) -> Option<LoadedRule> {
	for dir in dirs {
		let path = dir.join(format!("{}.toml", command));
		if path.is_file() {
			return load_path(path);
		}
	}

	for dir in dirs {
		let entries = match std::fs::read_dir(dir) {
			Ok(entries) => entries,
			Err(_) => continue,
		};
		let mut paths = entries
			.filter_map(|entry| entry.ok())
			.map(|entry| entry.path())
			.filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
			.collect::<Vec<PathBuf>>();
		paths.sort();

		for path in paths {
			let rule = match parse_file(&path) {
				Ok(rule) => rule,
				Err(err) => {
					eprintln!("runtime-rules: Failed to parse: {}", err);
					continue;
				}
			};
			if rule.commands.iter().any(|item| item == command) {
				return Some(LoadedRule { path, rule });
			}
		}
	}

	None
}

fn load_path(path: PathBuf) -> Option<LoadedRule> {
	match parse_file(&path) {
		Ok(rule) => Some(LoadedRule { path, rule }),
		Err(err) => {
			eprintln!("runtime-rules: Failed to parse: {}", err);
			None
		}
	}
}

fn base_dirs() -> Vec<PathBuf> {
	let mut dirs = Vec::new();

	#[cfg(windows)]
	{
		let config_home = std::env::var("APPDATA").unwrap_or_default();
		dirs.push(PathBuf::from(config_home));
	}

	#[cfg(not(windows))]
	{
		let config_home = std::env::var("XDG_CONFIG_HOME")
			.unwrap_or_else(|_| std::env::var("HOME").unwrap_or_default() + "/.config");
		dirs.push(PathBuf::from(config_home));

		let config_dirs =
			std::env::var("XDG_CONFIG_DIRS").unwrap_or_else(|_| "/etc/xdg".to_string());
		dirs.extend(config_dirs.split(':').map(PathBuf::from));

		let data_dirs = std::env::var("XDG_DATA_DIRS")
			.unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());
		dirs.extend(data_dirs.split(':').map(PathBuf::from));
	}

	dirs
}

pub fn rule_dirs() -> Vec<PathBuf> {
	base_dirs()
		.into_iter()
		.map(|dir| dir.join("pay-respects").join("rules"))
		.collect()
}

pub fn script_dirs() -> Vec<PathBuf> {
	base_dirs()
		.into_iter()
		.map(|dir| dir.join("pay-respects").join("scripts"))
		.collect()
}
