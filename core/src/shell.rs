use askama::Template;
use pay_respects_utils::lists::{alias_skip_expand, blocking_commands, privilege_list};
use pay_respects_utils::log::dlog;

use std::process::{Stdio, exit};

use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

#[cfg(windows)]
use pay_respects_utils::files::path_convert;

use crate::data::Data;
use crate::init::Init;
use crate::integrations::get_error_from_multiplexer;
use pay_respects_utils::remove_env_var;

/// Run the command without any shell configuration files (noprofile, norc)
fn clean_shell_command(shell: &str, command: &str) -> std::process::Command {
	let mut cmd = std::process::Command::new(shell);
	match shell {
		"bash" => {
			cmd.arg("--noprofile").arg("--norc").arg("-c").arg(command);
		}
		"zsh" => {
			cmd.arg("--no-rcs").arg("-c").arg(command);
		}
		"fish" => {
			cmd.arg("--no-config").arg("-c").arg(command);
		}
		"pwsh" | "powershell" => {
			cmd.arg("-NoProfile").arg("-c").arg(command);
		}
		"nu" => {
			cmd.arg("--no-config-file").arg("-c").arg(command);
		}
		_ => {
			cmd.arg("-c").arg(command);
		}
	}
	cmd
}

pub fn elevate(data: &mut Data, command: &mut String) {
	let first_command = command.split_whitespace().next().unwrap_or("");
	if is_privileged(data, first_command) {
		return;
	}
	if let Some(privilege) = &data.config.privilege {
		*command = format!("{} {}", privilege, command);
		return;
	}
	for privilege in privilege_list().iter() {
		if data.executables.contains(&privilege.to_string()) {
			*command = format!("{} {}", privilege, command);
			break;
		}
	}
}

pub fn is_privileged(data: &Data, command: &str) -> bool {
	if let Some(privilege) = &data.config.privilege {
		return privilege == command;
	}
	privilege_list().contains(&command)
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
			eprintln!("  - '{}'", candidate.trim());
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
		remove_env_var!("_PR_ERROR_MSG");
		return error_msg;
	} else {
		let timeout = data.config.timeout;
		#[cfg(debug_assertions)]
		eprintln!("timeout: {}", timeout);

		let executable = data.get_executable();
		if executable.is_empty() {
			return String::new();
		}
		if data.executables.contains(&executable.to_string()) {
			if let Some(unrunnable) = &data.config.blocking_commands
				&& unrunnable.contains(&executable.to_string())
			{
				return String::new();
			}
			if blocking_commands().contains(&executable) {
				return String::new();
			}
		}
		if let Some(error) =
			get_error_from_multiplexer(shell, &data.prompt_prefix, &data.input_command)
		{
			let message = format!("Captured output from multiplexer: '{}'", error);
			dlog(5, &message);
			error
		} else {
			error_output_threaded(shell, command, timeout)
		}
	};
	error.split_whitespace().collect::<Vec<&str>>().join(" ")
}

pub fn error_output_threaded(shell: &str, command: &str, timeout: u64) -> String {
	let (sender, receiver) = channel();

	thread::scope(|s| {
		s.spawn(|| {
			sender
				.send(
					clean_shell_command(shell, command)
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
	let output = clean_shell_command(shell, command)
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
	let output = clean_shell_command(shell, command)
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
	let mut last_command = data.command.clone();
	let comments = data.comments.clone();
	let error_msg = &data.error;
	let executables = {
		let exes = data.executables.clone().join(" ");
		if exes.len() < 100_000 {
			exes
		} else {
			"".to_string()
		}
	};

	if let Some(comments) = comments {
		last_command = format!("{} # {}", last_command, comments);
	}

	let output = clean_shell_command(shell, module)
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
		"pwsh" | "powershell" | "ps" => {
			for line in env.lines() {
				if !line.starts_with("Alias ") {
					continue;
				}
				let line = line.replacen("Alias ", "", 1);
				let (alias, command) = line.split_once("->").unwrap();
				let command = command.split_whitespace().next().unwrap_or("");
				alias_map.insert(alias.trim().to_string(), command.trim().to_string());
			}
		}
		"nu" | _ => {
			for line in env.lines() {
				let (alias, command) = line.split_once('=').unwrap();
				alias_map.insert(alias.to_string(), command.to_string());
			}
		}
	}
	remove_env_var!("_PR_ALIAS");
	Some(alias_map)
}

pub fn expand_alias(map: &HashMap<String, String>, command: &str) -> Option<String> {
	let (command, args) = if let Some(split) = command.split_once(' ') {
		(split.0, split.1)
	} else {
		(command, "")
	};
	if alias_skip_expand().contains(&command) {
		return None;
	}
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
	if command.contains("&")
		|| command.contains("|")
		|| command.contains('>')
		|| command.contains('|')
		|| command.contains('&')
		|| command.contains(';')
	{
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

	let prefixes = vec!["cd ", "Set-Location "];

	let cd = if success {
		let dirs = {
			let mut dirs = Vec::new();
			for line in lines {
				for prefix in &prefixes {
					if let Some(dir) = line.strip_prefix(prefix) {
						dirs.push(dir.to_string());
						break;
					}
				}
			}
			dirs.join("")
		};
		if dirs.is_empty() {
			None
		} else if shell == "nu" {
			Some(
				dirs.trim_matches('`')
					.trim_matches('"')
					.trim_matches('\'')
					.to_string(),
			)
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
		command: &'a str,
		cd: &'a str,
	}
	#[derive(Template)]
	#[template(path = "eval.ps1", escape = "none")]
	struct PwshTemplate<'a> {
		command: &'a str,
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
			// JSON-escape the fields so init.nu can parse the output with `from json`
			let escape_json = |s: &str| {
				s.replace('\\', "\\\\")
					.replace('"', "\\\"")
					.replace('\n', "\\n")
					.replace('\r', "\\r")
					.replace('\t', "\\t")
			};
			let command = escape_json(command);
			let cd_escaped = cd.as_deref().map(escape_json).unwrap_or_default();
			let template = NuTemplate {
				command: &command,
				cd: &cd_escaped,
			};
			template.render().unwrap()
		}
		"pwsh" | "powershell" | "ps" => {
			// Single-quoted PowerShell string: only ' needs escaping (doubled).
			let command = command.replace('\'', "''");
			let template = PwshTemplate {
				command: &command,
				cd: cd.as_deref(),
			};
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
