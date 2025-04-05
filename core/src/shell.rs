use pay_respects_utils::evals::split_command;
use pay_respects_utils::files::get_path_files;
use pay_respects_utils::files::path_env_sep;

use askama::Template;

use std::process::{exit, Stdio};

use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use pay_respects_utils::files::path_convert;

pub const PRIVILEGE_LIST: [&str; 2] = ["sudo", "doas"];

#[derive(PartialEq)]
pub enum Mode {
	Suggestion,
	Echo,
	Cnf,
}
pub struct Init {
	pub shell: String,
	pub binary_path: String,
	pub alias: String,
	pub cnf: bool,
}

impl Init {
	pub fn new() -> Init {
		Init {
			shell: String::from(""),
			binary_path: String::from(""),
			alias: String::from("f"),
			cnf: true,
		}
	}
}

pub struct Data {
	pub shell: String,
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
	pub mode: Mode,
}

impl Data {
	pub fn init() -> Data {
		let shell = get_shell();
		let command = last_command(&shell).trim().to_string();
		let alias = alias_map(&shell);
		let mode = run_mode();
		let (executables, modules, fallbacks);
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

		let mut init = Data {
			shell,
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
			mode,
		};

		init.split();
		init.expand_command();
		if init.mode != Mode::Cnf {
			init.update_error(None);
		}

		#[cfg(debug_assertions)]
		{
			eprintln!("/// data initialization");
			eprintln!("shell: {}", init.shell);
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
		let mut split = split_command(&self.command);
		if PRIVILEGE_LIST.contains(&split[0].as_str()) {
			self.command = self.command.replacen(&split[0], "", 1).trim().to_string();
			self.privilege = Some(split.remove(0))
		}
		#[cfg(debug_assertions)]
		eprintln!("split: {:?}", split);
		self.split = split;
	}

	pub fn update_error(&mut self, error: Option<String>) {
		if let Some(error) = error {
			self.error = error
				.to_lowercase()
				.split_whitespace()
				.collect::<Vec<&str>>()
				.join(" ");
		} else {
			self.error = get_error(&self.shell, &self.command);
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

pub fn elevate(data: &mut Data, command: &mut String) {
	for privilege in PRIVILEGE_LIST.iter() {
		if data.executables.contains(&privilege.to_string()) {
			*command = format!("{} {}", privilege, command);
			break;
		}
	}
}

pub fn add_candidates_no_dup(
	command: &str,
	candidates: &mut Vec<String>,
	new_candidates: &[String],
) {
	for candidate in new_candidates {
		let candidate = candidate.trim();
		if candidate != command && !candidates.contains(&candidate.to_string()) {
			candidates.push(candidate.to_string());
		}
	}
}

pub fn get_error(shell: &str, command: &str) -> String {
	let error_msg = std::env::var("_PR_ERROR_MSG");
	let error = if let Ok(error_msg) = error_msg {
		std::env::remove_var("_PR_ERROR_MSG");
		error_msg
	} else {
		error_output_threaded(shell, command)
	};
	error
		.to_lowercase()
		.split_whitespace()
		.collect::<Vec<&str>>()
		.join(" ")
}

pub fn error_output_threaded(shell: &str, command: &str) -> String {
	let (sender, receiver) = channel();

	thread::scope(|s| {
		s.spawn(|| {
			sender
				.send(
					std::process::Command::new(shell)
						.arg("-c")
						.arg(command)
						.env("LC_ALL", "C")
						.output()
						.expect("failed to execute process"),
				)
				.expect("failed to send output");
		});

		match receiver.recv_timeout(Duration::from_secs(3)) {
			Ok(output) => match output.stderr.is_empty() {
				true => String::from_utf8_lossy(&output.stdout).to_string(),
				false => String::from_utf8_lossy(&output.stderr).to_string(),
			},
			Err(_) => {
				use colored::*;
				eprintln!("Timeout while executing command: {}", command.red());
				exit(1);
			}
		}
	})
}

pub fn command_output(shell: &str, command: &str) -> String {
	let output = std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		.env("LC_ALL", "C")
		.output()
		.expect("failed to execute process");

	String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn module_output(data: &Data, module: &str) -> Option<Vec<String>> {
	let shell = &data.shell;
	let executable = &data.split[0];
	let last_command = &data.command;
	let error_msg = &data.error;
	let executables = {
		let exes = data.executables.clone().join(" ");
		if exes.len() < 100_000 {
			exes
		} else {
			"".to_string()
		}
	};
	let output = std::process::Command::new(shell)
		.arg("-c")
		.arg(module)
		.env("_PR_COMMAND", executable)
		.env("_PR_SHELL", shell)
		.env("_PR_LAST_COMMAND", last_command)
		.env("_PR_ERROR_MSG", error_msg)
		.env("_PR_EXECUTABLES", executables)
		.stderr(Stdio::inherit())
		.output()
		.expect("failed to execute process");

	if output.stdout.is_empty() {
		return None;
	}
	let break_holder = "<_PR_BR>";
	Some(
		String::from_utf8_lossy(&output.stdout)[..output.stdout.len() - break_holder.len()]
			.split("<_PR_BR>")
			.map(|s| s.trim().to_string())
			.collect::<Vec<String>>(),
	)
}

pub fn last_command(shell: &str) -> String {
	let last_command = match std::env::var("_PR_LAST_COMMAND") {
		Ok(command) => command,
		Err(_) => {
			eprintln!(
				"{}",
				t!(
					"no-env-setup",
					var = "_PR_LAST_COMMAND",
					help = "pay-respects -h"
				)
			);
			exit(1);
		}
	};

	match shell {
		"bash" => last_command,
		"zsh" => last_command,
		"fish" => last_command,
		"nu" => last_command,
		_ => last_command,
	}
}

pub fn run_mode() -> Mode {
	match std::env::var("_PR_MODE") {
		Ok(mode) => match mode.as_str() {
			"suggestion" => Mode::Suggestion,
			"cnf" => Mode::Cnf,
			"echo" => Mode::Echo,
			_ => {
				eprintln!("Invalid mode: {}", mode);
				exit(1);
			}
		},
		Err(_) => Mode::Suggestion,
	}
}

#[allow(clippy::wildcard_in_or_patterns)]
pub fn alias_map(shell: &str) -> Option<HashMap<String, String>> {
	let env = std::env::var("_PR_ALIAS");

	if env.is_err() {
		return None;
	}
	let env = env.unwrap();
	if env.is_empty() {
		return None;
	}

	let mut alias_map = HashMap::new();
	match shell {
		"bash" => {
			for line in env.lines() {
				let alias = line.replace("alias ", "");
				let (alias, command) = alias.split_once('=').unwrap();
				let command = command.trim().trim_matches('\'');
				alias_map.insert(alias.to_string(), command.to_string());
			}
		}
		"zsh" => {
			for line in env.lines() {
				let (alias, command) = line.split_once('=').unwrap();
				let command = command.trim().trim_matches('\'');
				alias_map.insert(alias.to_string(), command.to_string());
			}
		}
		"fish" => {
			for line in env.lines() {
				let alias = line.replace("alias ", "");
				let (alias, command) = alias.split_once(' ').unwrap();
				let command = command.trim().trim_matches('\'');
				alias_map.insert(alias.to_string(), command.to_string());
			}
		}
		"nu" | _ => {
			for line in env.lines() {
				let (alias, command) = line.split_once('=').unwrap();
				alias_map.insert(alias.to_string(), command.to_string());
			}
		}
	}
	std::env::remove_var("_PR_ALIAS");
	Some(alias_map)
}

pub fn expand_alias(map: &HashMap<String, String>, command: &str) -> Option<String> {
	let (command, args) = if let Some(split) = command.split_once(' ') {
		(split.0, split.1)
	} else {
		(command, "")
	};
	map.get(command)
		.map(|expand| format!("{} {}", expand, args))
}

pub fn expand_alias_multiline(map: &HashMap<String, String>, command: &str) -> Option<String> {
	let lines = command.lines().collect::<Vec<&str>>();
	let mut expanded = String::new();
	let mut expansion = false;
	for line in lines {
		if let Some(expand) = expand_alias(map, line) {
			expanded = format!("{}\n{}", expanded, expand);
			expansion = true;
		} else {
			expanded = format!("{}\n{}", expanded, line);
		}
	}
	if expansion {
		Some(expanded.trim().to_string())
	} else {
		None
	}
}

pub fn initialization(init: &mut Init) {
	let alias = &init.alias;
	let cnf = init.cnf;
	let binary_path = &init.binary_path;

	let shell = &init.shell;

	#[derive(Template)]
	#[template(path = "init.bash", escape = "none")]
	struct BashTemplate<'a> {
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
	}
	#[derive(Template)]
	#[template(path = "init.zsh", escape = "none")]
	struct ZshTemplate<'a> {
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
	}
	#[derive(Template)]
	#[template(path = "init.fish", escape = "none")]
	struct FishTemplate<'a> {
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
	}
	#[derive(Template)]
	#[template(path = "init.ps1", escape = "none")]
	struct PowershellTemplate<'a> {
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
	}
	#[derive(Template)]
	#[template(path = "init.nu", escape = "none")]
	struct NuTemplate<'a> {
		alias: &'a str,
		binary_path: &'a str,
	}

	let initialize = match shell.as_str() {
		"bash" => BashTemplate {
			alias,
			binary_path,
			cnf,
		}
		.render()
		.unwrap(),
		"zsh" => ZshTemplate {
			alias,
			binary_path,
			cnf,
		}
		.render()
		.unwrap(),
		"fish" => FishTemplate {
			alias,
			binary_path,
			cnf,
		}
		.render()
		.unwrap(),
		"pwsh" | "powershell" | "ps" => PowershellTemplate {
			alias,
			binary_path,
			cnf,
		}
		.render()
		.unwrap(),
		"nu" | "nush" | "nushell" => NuTemplate { alias, binary_path }.render().unwrap(),
		_ => {
			eprintln!("{}: {}", t!("unknown-shell"), shell);
			exit(1);
		}
	};

	println!("{}", initialize);
}

pub fn get_shell() -> String {
	match std::env::var("_PR_SHELL") {
		Ok(shell) => shell,
		Err(_) => {
			eprintln!(
				"{}",
				t!("no-env-setup", var = "_PR_SHELL", help = "pay-respects -h")
			);
			std::process::exit(1);
		}
	}
}

pub fn shell_syntax(shell: &str, command: &str) -> String {
	#[allow(clippy::single_match)]
	match shell {
		"nu" => command.replace("&&\n", ";\n").to_string(),
		_ => command.to_string(),
	}
}

pub fn shell_evaluated_commands(shell: &str, command: &str) -> Option<String> {
	let lines = command
		.lines()
		.map(|line| line.trim().trim_end_matches(['\\', ';', '|', '&']))
		.collect::<Vec<&str>>();
	let mut dirs = Vec::new();
	for line in lines {
		if let Some(dir) = line.strip_prefix("cd ") {
			dirs.push(dir.to_string());
		}
	}

	let cd_dir = dirs.join("");
	if cd_dir.is_empty() {
		return None;
	}

	#[allow(clippy::single_match)]
	match shell {
		"nu" => Some(cd_dir),
		_ => Some(format!("cd {}", cd_dir)),
	}
}
