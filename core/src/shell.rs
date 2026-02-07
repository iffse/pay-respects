use askama::Template;

use std::process::{exit, Stdio};

use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use pay_respects_utils::files::path_convert;

use crate::data::{Data, Mode};
use crate::init::Init;

const PRIVILEGE_LIST: [&str; 2] = ["sudo", "doas"];

pub fn elevate(data: &mut Data, command: &mut String) {
	if is_privileged(command, data) {
		return;
	}
	if data.config.sudo.is_some() {
		*command = format!("{} {}", data.config.sudo.as_ref().unwrap(), command);
		return;
	}
	for privilege in PRIVILEGE_LIST.iter() {
		if data.executables.contains(&privilege.to_string()) {
			*command = format!("{} {}", privilege, command);
			break;
		}
	}
}

pub fn is_privileged(command: &str, data: &Data) -> bool {
	if data.config.sudo.is_some() {
		return command == data.config.sudo.as_ref().unwrap();
	}
	PRIVILEGE_LIST.contains(&command)
}

pub fn add_candidates_no_dup(
	command: &str,
	candidates: &mut Vec<String>,
	new_candidates: &[String],
) {
	#[cfg(debug_assertions)]
	{
		eprintln!("Adding candidates for command: '{}'", command);
		for candidate in new_candidates {
			eprintln!("  - '{}'", candidate);
		}
	}
	for candidate in new_candidates {
		let candidate = candidate.trim();
		if candidate.is_empty() {
			continue;
		}
		if candidate != command && !candidates.contains(&candidate.to_string()) {
			candidates.push(candidate.to_string());
		}
	}
}

pub fn get_error(shell: &str, command: &str, data: &Data) -> String {
	let error_msg = std::env::var("_PR_ERROR_MSG");
	let error = if let Ok(error_msg) = error_msg {
		std::env::remove_var("_PR_ERROR_MSG");
		error_msg
	} else {
		let timeout = data.config.timeout;
		#[cfg(debug_assertions)]
		eprintln!("timeout: {}", timeout);
		error_output_threaded(shell, command, timeout)
	};
	error
		.to_lowercase()
		.split_whitespace()
		.collect::<Vec<&str>>()
		.join(" ")
}

pub fn error_output_threaded(shell: &str, command: &str, timeout: u64) -> String {
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

		match receiver.recv_timeout(Duration::from_millis(timeout)) {
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

	let err = String::from_utf8_lossy(&output.stderr);
	if !err.is_empty() {
		eprintln!("Error while executing command: {}", command);
		eprintln!("  {}", err.replace("\n", "\n  "));
	}

	String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn command_output_or_error(shell: &str, command: &str) -> String {
	let output = std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		.env("LC_ALL", "C")
		.output()
		.expect("failed to execute process");

	if !output.stdout.is_empty() {
		String::from_utf8_lossy(&output.stdout).to_string()
	} else {
		String::from_utf8_lossy(&output.stderr).to_string()
	}
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
		String::from_utf8_lossy(&output.stdout)
			.trim_end_matches(break_holder)
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
			"noconfirm" => Mode::NoConfirm,
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
			// fix for multiline aliases
			let lines = env.split("\nalias ").collect::<Vec<&str>>().into_iter();
			for line in lines {
				let (alias, command) = line.split_once('=').unwrap();
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

#[allow(unused_variables)]
pub fn builtin_commands(shell: &str) -> Vec<String> {
	// TODO: add the commands for each shell
	// these should cover most of the builtin commands
	// (maybe with false positives)
	let builtin = vec![
		"alias", "bg", "bind", "break", "builtin", "case", "cd", "command", "compgen", "complete",
		"continue", "declare", "dirs", "disown", "echo", "enable", "eval", "exec", "exit",
		"export", "fc", "fg", "getopts", "hash", "help", "history", "if", "jobs", "kill", "let",
		"local", "logout", "popd", "printf", "pushd", "pwd", "read", "readonly", "return", "set",
		"shift", "shopt", "source", "suspend", "test", "times", "trap", "type", "typeset",
		"ulimit", "umask", "unalias", "unset", "until", "wait", "while", "which",
	];
	builtin.iter().map(|&cmd| cmd.to_string()).collect()
}

pub fn shell_syntax(shell: &str, command: &str) -> String {
	#[allow(clippy::single_match)]
	match shell {
		"nu" => command.replace("&&\n", ";\n").to_string(),
		_ => command.to_string(),
	}
}

pub fn add_privilege(shell: &str, privilege: &str, command: &str) -> String {
	if command.contains("&&") || command.contains("||") || command.contains('>') {
		format!(
			"{} {} -c \"{}\"",
			privilege,
			shell,
			command.replace("\"", "\\\"")
		)
	} else {
		format!("{} {}", privilege, command)
	}
}

pub fn shell_evaluated_commands(shell: &str, command: &str, success: bool) {
	let lines = command
		.lines()
		.map(|line| line.trim().trim_end_matches(['\\', ';', '|', '&']))
		.collect::<Vec<&str>>();

	let cd = if success {
		let dirs = {
			let mut dirs = Vec::new();
			for line in lines {
				if let Some(dir) = line.strip_prefix("cd ") {
					dirs.push(dir.to_string());
				}
			}
			dirs.join("")
		};
		if dirs.is_empty() {
			None
		} else {
			Some(dirs.to_string())
		}
	} else {
		None
	};

	#[derive(Template)]
	#[template(path = "eval.bash", escape = "none")]
	struct BashTemplate<'a> {
		command: &'a str,
		cd: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "eval.zsh", escape = "none")]
	struct ZshTemplate<'a> {
		command: &'a str,
		cd: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "eval.fish", escape = "none")]
	struct FishTemplate<'a> {
		command: &'a str,
		cd: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "eval.nu", escape = "none")]
	struct NuTemplate<'a> {
		cd: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "eval.sh", escape = "none")]
	struct GenericTemplate<'a> {
		cd: Option<&'a str>,
	}

	let print = match shell {
		"bash" => {
			let command = command
				.replace("$", "\\$")
				.replace("`", "\\`")
				.replace("\"", "\\\"");
			let template = BashTemplate {
				command: &command,
				cd: cd.as_deref(),
			};
			template.render().unwrap()
		}
		"zsh" => {
			let command = command
				.replace("$", "\\$")
				.replace("`", "\\`")
				.replace("\"", "\\\"");
			let template = ZshTemplate {
				command: &command,
				cd: cd.as_deref(),
			};
			template.render().unwrap()
		}
		"fish" => {
			let command = command
				.replace("$", "\\$")
				.replace("`", "\\`")
				.replace("\"", "\\\"");
			let template = FishTemplate {
				command: &command,
				cd: cd.as_deref(),
			};
			template.render().unwrap()
		}
		"nu" => {
			let template = NuTemplate { cd: cd.as_deref() };
			template.render().unwrap()
		}
		_ => {
			let template = GenericTemplate { cd: cd.as_deref() };
			template.render().unwrap()
		}
	};
	let print = print.trim();
	if !print.is_empty() {
		println!("{}", print);
	}
}
