use pay_respects_utils::evals::split_command;
use pay_respects_utils::files::get_path_files;
use pay_respects_utils::files::path_env_sep;

use itertools::Itertools;

use std::process::exit;

use std::collections::HashMap;

#[cfg(windows)]
use pay_respects_utils::files::path_convert;

use crate::config::load_config;
use crate::config::Config;
use crate::shell::alias_map;
use crate::shell::builtin_commands;
use crate::shell::expand_alias_multiline;
use crate::shell::get_error;
use crate::shell::get_shell;
use crate::shell::last_command;
use crate::shell::run_mode;

pub const PRIVILEGE_LIST: [&str; 2] = ["sudo", "doas"];

#[derive(PartialEq)]
pub enum Mode {
	Suggestion,
	Echo,
	NoConfirm,
	Cnf,
}
pub struct Data {
	pub shell: String,
	pub env: Option<String>,
	pub command: String,
	pub suggest: Option<String>,
	pub candidates: Vec<String>,
	pub split: Vec<String>,
	pub alias: Option<HashMap<String, String>>,
	pub privilege: Option<String>,
	pub error: String,
	pub executables: Vec<String>,
	pub modules: Vec<String>,
	pub fallbacks: Vec<String>,
	pub config: Config,
	pub mode: Mode,
}

impl Data {
	pub fn init() -> Data {
		let shell = get_shell();
		let command = last_command(&shell).trim().to_string();
		let alias = alias_map(&shell);
		let mode = run_mode();
		let (mut executables, modules, fallbacks);
		let lib_dir = {
			if let Ok(lib_dir) = std::env::var("_PR_LIB") {
				Some(lib_dir)
			} else {
				option_env!("_DEF_PR_LIB").map(|dir| dir.to_string())
			}
		};

		#[cfg(debug_assertions)]
		eprintln!("lib_dir: {:?}", lib_dir);

		if lib_dir.is_none() {
			(executables, modules, fallbacks) = {
				let path_executables = get_path_files();
				let mut executables = vec![];
				let mut modules = vec![];
				let mut fallbacks = vec![];
				for exe in path_executables {
					if exe.starts_with("_pay-respects-module-") {
						modules.push(exe.to_string());
					} else if exe.starts_with("_pay-respects-fallback-") {
						fallbacks.push(exe.to_string());
					} else {
						executables.push(exe.to_string());
					}
				}
				modules.sort_unstable();
				fallbacks.sort_unstable();
				if alias.is_some() {
					let alias = alias.as_ref().unwrap();
					for command in alias.keys() {
						if executables.contains(command) {
							continue;
						}
						executables.push(command.to_string());
					}
				}

				(executables, modules, fallbacks)
			};
		} else {
			(executables, modules, fallbacks) = {
				let mut modules = vec![];
				let mut fallbacks = vec![];
				let lib_dir = lib_dir.unwrap();
				let mut executables = get_path_files();
				if alias.is_some() {
					let alias = alias.as_ref().unwrap();
					for command in alias.keys() {
						if executables.contains(command) {
							continue;
						}
						executables.push(command.to_string());
					}
				}

				let path = lib_dir.split(path_env_sep()).collect::<Vec<&str>>();

				for p in path {
					#[cfg(windows)]
					let p = path_convert(p);

					let files = match std::fs::read_dir(p) {
						Ok(files) => files,
						Err(_) => continue,
					};
					for file in files {
						let file = file.unwrap();
						let file_name = file.file_name().into_string().unwrap();
						let file_path = file.path();

						if file_name.starts_with("_pay-respects-module-") {
							modules.push(file_path.to_string_lossy().to_string());
						} else if file_name.starts_with("_pay-respects-fallback-") {
							fallbacks.push(file_path.to_string_lossy().to_string());
						}
					}
				}

				modules.sort_unstable();
				fallbacks.sort_unstable();

				(executables, modules, fallbacks)
			};
		}

		let builtins = builtin_commands(&shell);
		executables.extend(builtins.clone());
		executables = executables.iter().unique().cloned().collect();
		let config = load_config();

		let mut init = Data {
			shell,
			env: None,
			command,
			suggest: None,
			candidates: vec![],
			alias,
			split: vec![],
			privilege: None,
			error: "".to_string(),
			executables,
			modules,
			fallbacks,
			config,
			mode,
		};

		init.split();
		init.extract_env();
		init.expand_command();
		if init.mode != Mode::Cnf {
			init.update_error(None);
		}

		#[cfg(debug_assertions)]
		{
			eprintln!("/// data initialization");
			eprintln!("shell: {}", init.shell);
			eprintln!("env: {:?}", init.env);
			eprintln!("command: {}", init.command);
			eprintln!("error: {}", init.error);
			eprintln!("modules: {:?}", init.modules);
			eprintln!("fallbacks: {:?}", init.fallbacks);
		}

		init
	}

	pub fn expand_command(&mut self) {
		if self.alias.is_none() {
			return;
		}
		let alias = self.alias.as_ref().unwrap();
		if let Some(command) = expand_alias_multiline(alias, &self.command) {
			#[cfg(debug_assertions)]
			eprintln!("expand_command: {}", command);
			self.update_command(&command);
		}
	}

	pub fn expand_suggest(&mut self) {
		if self.alias.is_none() {
			return;
		}
		let alias = self.alias.as_ref().unwrap();
		if let Some(suggest) = expand_alias_multiline(alias, self.suggest.as_ref().unwrap()) {
			#[cfg(debug_assertions)]
			eprintln!("expand_suggest: {}", suggest);
			self.update_suggest(&suggest);
		}
	}

	pub fn split(&mut self) {
		self.extract_privilege();
		let split = split_command(&self.command);
		#[cfg(debug_assertions)]
		eprintln!("split: {:?}", split);
		if split.is_empty() {
			eprintln!("{}", t!("empty-command"));
			exit(1);
		}
		self.split = split;
	}

	pub fn extract_privilege(&mut self) {
		let command = {
			let first = self.command.split_whitespace().next();
			if let Some(first) = first {
				first.to_string()
			} else {
				return;
			}
		};
		if let Some(sudo) = self.config.sudo.as_ref() {
			if command == *sudo {
				self.privilege = Some(command.to_string());
				self.command = self.command.replacen(sudo, "", 1).trim().to_string();
			}
			return;
		}
		if PRIVILEGE_LIST.contains(&command.as_str()) {
			self.privilege = Some(command.to_string());
			let sudo = split_command(&self.command)[0].clone();
			self.command = self.command.replacen(&sudo, "", 1).trim().to_string();
		}
	}

	pub fn extract_env(&mut self) {
		let mut envs = vec![];
		loop {
			let mut char = self.split[0].char_indices();
			char.next();
			let offset = char.offset();
			if self.split[0][offset..].contains("=") {
				envs.push(self.split.remove(0));
			} else {
				break;
			}
		}
		if !envs.is_empty() {
			self.env = Some(envs.join(" "));
			self.command = self.split.join(" ");
		}
	}

	pub fn update_error(&mut self, error: Option<String>) {
		if let Some(error) = error {
			self.error = error
				.to_lowercase()
				.split_whitespace()
				.collect::<Vec<&str>>()
				.join(" ");
		} else {
			self.error = get_error(&self.shell, &self.command, self);
		}
	}

	pub fn update_command(&mut self, command: &str) {
		self.command = command.to_string();
		self.split();
	}

	pub fn update_suggest(&mut self, suggest: &str) {
		let split = split_command(suggest);
		if PRIVILEGE_LIST.contains(&split[0].as_str()) {
			self.suggest = Some(suggest.replacen(&split[0], "", 1));
			self.privilege = Some(split[0].clone())
		} else {
			self.suggest = Some(suggest.to_string());
		};
	}
}
