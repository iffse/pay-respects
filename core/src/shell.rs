use askama::Template;
use pay_respects_utils::lists::{alias_skip_expand, blocking_commands, privilege_list};
use pay_respects_utils::shell::{
	ShellType, detect_shell_type, get_shell_type as get_global_shell_type, normalize_shell_type,
};
use pay_respects_utils::strings::print_error;

use std::process::{Stdio, exit};

use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use crate::data::Data;
use crate::init::Init;
use crate::integrations::get_error_from_multiplexer;
use pay_respects_utils::remove_env_var;

/// Run the command without any shell configuration files (noprofile, norc)
fn clean_shell_command(shell: &str, command: &str) -> std::process::Command {
	let mut cmd = std::process::Command::new(shell);
	match get_global_shell_type() {
		ShellType::Bash => {
			cmd.arg("--noprofile").arg("--norc").arg("-c").arg(command);
		}
		ShellType::Zsh => {
			cmd.arg("--no-rcs").arg("-c").arg(command);
		}
		ShellType::Fish => {
			cmd.arg("--no-config").arg("-c").arg(command);
		}
		ShellType::Dash => {
			cmd.arg("-c").arg(command);
		}
		ShellType::Xonsh => {
			cmd.arg("--no-rc").arg("-c").arg(command);
		}
		ShellType::Elvish => {
			cmd.arg("-norc").arg("-c").arg(command);
		}
		ShellType::Powershell => {
			// omg powershell
			let invariant_culture_command = format!(
				"$CurrentCulture = [System.Globalization.CultureInfo]::InvariantCulture; $CurrentUICulture = [System.Globalization.CultureInfo]::InvariantCulture; {}",
				command
			);
			cmd.arg("-NoProfile")
				.arg("-c")
				.arg(invariant_culture_command);
		}
		ShellType::Nu => {
			cmd.arg("--no-config-file").arg("-c").arg(command);
		}
		ShellType::Generic => {
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
	if new_candidates.is_empty() {
		return;
	}
	pay_respects_utils::dlog!(2, "Adding candidates for command: '{}'", command);
	#[cfg(debug_assertions)]
	for candidate in new_candidates {
		pay_respects_utils::dlog!(2, "  - '{}'", candidate.trim());
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
		pay_respects_utils::dlog!(2, "timeout: {}", timeout);

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
			pay_respects_utils::dlog!(3, "Captured output from multiplexer: '{}'", error);
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
						.unwrap_or_else(|_| {
							panic!(
								"failed to execute process, is '{}' the correct executable?",
								shell
							)
						}),
				)
				.expect("failed to send output");
		});

		match receiver.recv_timeout(Duration::from_millis(timeout)) {
			Ok(output) => match output.stderr.is_empty() {
				true => String::from_utf8_lossy(&output.stdout).to_string(),
				false => String::from_utf8_lossy(&output.stderr).to_string(),
			},
			Err(_) => {
				print_error(&format!("Timeout while executing command: {}", command));
				exit(1);
			}
		}
	})
}

pub fn command_output(shell: &str, command: &str) -> String {
	let output = clean_shell_command(shell, command)
		.env("LC_ALL", "C")
		.output()
		.unwrap_or_else(|_| {
			panic!(
				"failed to execute process, is '{}' the correct executable?",
				shell
			)
		});

	let err = String::from_utf8_lossy(&output.stderr);
	if !err.is_empty() {
		print_error(&format!(
			"Error while executing command: {}\n  {}",
			command,
			err.replace("\n", "\n  ")
		));
	}

	String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn command_output_or_error(shell: &str, command: &str) -> String {
	let output = clean_shell_command(shell, command)
		.env("LC_ALL", "C")
		.output()
		.unwrap_or_else(|_| {
			panic!(
				"failed to execute process, is '{}' the correct executable?",
				shell
			)
		});

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
		.env("_PR_SHELL_TYPE", &data.shell_type)
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
			print_error(&t!(
				"no-env-setup",
				var = "_PR_LAST_COMMAND",
				help = "pay-respects -h"
			));
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
		"dash" => {
			for line in env.lines() {
				let line = line.trim_matches('\'');
				if let Some((alias, command)) = line.split_once('=') {
					alias_map.insert(alias.to_string(), command.to_string());
				}
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
	let prefix = init.cmd_prefix.as_deref();

	let shell = &init.shell;
	let shell_type = &init.shell_type;

	#[derive(Template)]
	#[template(path = "init.bash", escape = "none")]
	struct BashTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
		prefix: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "init.zsh", escape = "none")]
	struct ZshTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
		prefix: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "init.fish", escape = "none")]
	struct FishTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
		prefix: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "init.dash", escape = "none")]
	struct DashTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		prefix: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "init.xonsh", escape = "none")]
	struct XonshTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
		prefix: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "init.elv", escape = "none")]
	struct ElvishTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		prefix: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "init.ps1", escape = "none")]
	struct PowershellTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		cnf: bool,
		prefix: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "init.nu", escape = "none")]
	struct NuTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		prefix: Option<&'a str>,
	}

	let initialize = match shell_type.as_str() {
		"bash" => BashTemplate {
			shell,
			shell_type,
			alias,
			binary_path,
			cnf,
			prefix,
		}
		.render()
		.unwrap(),
		"zsh" => ZshTemplate {
			shell,
			shell_type,
			alias,
			binary_path,
			cnf,
			prefix,
		}
		.render()
		.unwrap(),
		"fish" => FishTemplate {
			shell,
			shell_type,
			alias,
			binary_path,
			cnf,
			prefix,
		}
		.render()
		.unwrap(),
		"dash" => DashTemplate {
			shell,
			shell_type,
			alias,
			binary_path,
			prefix,
		}
		.render()
		.unwrap(),
		"xonsh" => XonshTemplate {
			shell,
			shell_type,
			alias,
			binary_path,
			cnf,
			prefix,
		}
		.render()
		.unwrap(),
		"elvish" => ElvishTemplate {
			shell,
			shell_type,
			alias,
			binary_path,
			prefix,
		}
		.render()
		.unwrap(),
		"pwsh" | "powershell" | "ps" => PowershellTemplate {
			shell,
			shell_type,
			alias,
			binary_path,
			cnf,
			prefix,
		}
		.render()
		.unwrap(),
		"nu" | "nush" | "nushell" => NuTemplate {
			shell,
			shell_type,
			alias,
			binary_path,
			prefix,
		}
		.render()
		.unwrap(),
		_ => {
			print_error(&format!("Unknown shell: {}", shell));
			exit(1);
		}
	};

	println!("{}", initialize);
}

pub fn get_shell() -> String {
	match std::env::var("_PR_SHELL") {
		Ok(shell) => shell,
		Err(_) => {
			print_error(&t!(
				"no-env-setup",
				var = "_PR_SHELL",
				help = "pay-respects -h"
			));
			std::process::exit(1);
		}
	}
}

pub fn get_shell_type(shell: &str) -> String {
	match std::env::var("_PR_SHELL_TYPE") {
		Ok(shell_type) if !shell_type.is_empty() => normalize_shell_type(&shell_type)
			.unwrap_or(&shell_type)
			.to_string(),
		_ => detect_shell_type(shell).unwrap_or(shell).to_string(),
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
		"nu" | "nush" | "nushell" | "xonsh" | "elvish" => {
			command.replace("&&\n", ";\n").to_string()
		}
		_ => command.to_string(),
	}
}

pub fn history_command(shell: &str, privilege: Option<&str>, command: &str) -> String {
	match privilege {
		Some(privilege) => add_privilege(shell, privilege, command),
		None => command.to_string(),
	}
}

pub fn add_privilege(shell: &str, privilege: &str, command: &str) -> String {
	if command
		.chars()
		.any(|character| matches!(character, '&' | '|' | '>' | '<' | ';' | '\n'))
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
	#[template(path = "eval.xonsh", escape = "none")]
	struct XonshEvalTemplate<'a> {
		command: &'a str,
		cd: Option<&'a str>,
	}
	#[derive(Template)]
	#[template(path = "eval.elv", escape = "none")]
	struct ElvishEvalTemplate<'a> {
		command: &'a str,
		cd: Option<&'a str>,
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
		"nu" | "nush" | "nushell" => {
			// JSON-escape the fields so init.nu can parse the output with `from json`
			let command = escape_json(command);
			let cd_escaped = cd.as_deref().map(escape_json).unwrap_or_default();
			let template = NuTemplate {
				command: &command,
				cd: &cd_escaped,
			};
			template.render().unwrap()
		}
		"xonsh" => {
			let escape = |s: &str| {
				s.replace('\\', "\\\\")
					.replace('"', "\\\"")
					.replace('\n', "\\n")
					.replace('\r', "\\r")
					.replace('\t', "\\t")
			};
			let command = escape(command);
			let cd_escaped = cd.as_deref().map(escape);
			let template = XonshEvalTemplate {
				command: &command,
				cd: cd_escaped.as_deref(),
			};
			template.render().unwrap()
		}
		"elvish" => {
			let command = command.replace('\'', "''");
			let cd_escaped = cd.as_deref().map(|cd| cd.replace('\'', "''"));
			let template = ElvishEvalTemplate {
				command: &command,
				cd: cd_escaped.as_deref(),
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

/// Shells that can put a command back into their prompt buffer.
pub fn prompt_insert_supported(shell_type: &str) -> bool {
	matches!(shell_type, "zsh" | "fish" | "nu")
}

/// Prints shell code that puts `command` back into the prompt without running
/// it, so the user can edit it with the shell's own line editor.
pub fn print_prompt_command(shell_type: &str, command: &str) {
	if let Some(print) = prompt_command(shell_type, command) {
		println!("{}", print);
	}
}

fn prompt_command(shell_type: &str, command: &str) -> Option<String> {
	#[derive(Template)]
	#[template(path = "prompt.zsh", escape = "none")]
	struct ZshPromptTemplate<'a> {
		command: &'a str,
	}
	#[derive(Template)]
	#[template(path = "prompt.fish", escape = "none")]
	struct FishPromptTemplate<'a> {
		command: &'a str,
	}
	#[derive(Template)]
	#[template(path = "prompt.nu", escape = "none")]
	struct NuPromptTemplate<'a> {
		command: &'a str,
	}

	let print = match shell_type {
		"zsh" => ZshPromptTemplate {
			command: &escape_double_quoted(command),
		}
		.render()
		.unwrap(),
		"fish" => FishPromptTemplate {
			command: &escape_double_quoted(command),
		}
		.render()
		.unwrap(),
		"nu" => NuPromptTemplate {
			command: &escape_json(command),
		}
		.render()
		.unwrap(),
		_ => return None,
	};

	Some(print.trim().to_string())
}

/// Escapes a command for a double-quoted shell string.
fn escape_double_quoted(command: &str) -> String {
	command
		.replace('\\', "\\\\")
		.replace('"', "\\\"")
		.replace('$', "\\$")
		.replace('`', "\\`")
}

/// JSON-escapes a command for the Nushell initialization.
fn escape_json(command: &str) -> String {
	command
		.replace('\\', "\\\\")
		.replace('"', "\\\"")
		.replace('\n', "\\n")
		.replace('\r', "\\r")
		.replace('\t', "\\t")
}

#[cfg(test)]
mod tests {
	use super::{history_command, prompt_command, prompt_insert_supported, shell_syntax};
	use askama::Template;

	#[derive(Template)]
	#[template(path = "init.elv", escape = "none")]
	struct ElvishInitTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		prefix: Option<&'a str>,
	}

	#[derive(Template)]
	#[template(path = "init.nu", escape = "none")]
	struct NuInitTemplate<'a> {
		shell: &'a str,
		shell_type: &'a str,
		alias: &'a str,
		binary_path: &'a str,
		prefix: Option<&'a str>,
	}

	#[test]
	fn nushell_init_ignores_its_own_history_entry() {
		let init = NuInitTemplate {
			shell: "nu",
			shell_type: "nu",
			alias: "f",
			binary_path: "pay-respects",
			prefix: None,
		}
		.render()
		.unwrap();

		assert!(init.contains("let alias = 'f'"));
		assert!(init.contains("($entry.command | str trim) != $alias"));
		assert!(!init.contains("(history | last).command"));
	}

	#[test]
	fn prompt_insert_is_only_offered_for_supported_shells() {
		for shell in ["zsh", "fish", "nu"] {
			assert!(prompt_insert_supported(shell));
		}
		for shell in ["bash", "dash", "pwsh", "xonsh", "elvish"] {
			assert!(!prompt_insert_supported(shell));
		}
	}

	#[test]
	fn prompt_command_targets_the_shell_and_escapes_the_command() {
		assert_eq!(
			prompt_command("zsh", "echo \"$HOME\" 1\\2").unwrap(),
			r#"print -z "echo \"\$HOME\" 1\\2""#
		);
		assert_eq!(
			prompt_command("fish", "git status").unwrap(),
			r#"commandline --replace "git status""#
		);
		assert_eq!(
			prompt_command("nu", "echo \"hi\"").unwrap(),
			r#"{ "cd": "", "command": "", "prompt": "echo \"hi\"" }"#
		);
		assert!(prompt_command("bash", "echo hi").is_none());
		assert!(prompt_command("elvish", "echo hi").is_none());
	}

	#[test]
	fn elvish_init_exports_alias_from_eval() {
		let init = ElvishInitTemplate {
			shell: "elvish",
			shell_type: "elvish",
			alias: "f",
			binary_path: "pay-respects",
			prefix: None,
		}
		.render()
		.unwrap();

		assert!(init.contains("edit:add-var f~ $_pr_suggest"));
		assert!(init.contains("set edit:insert:binding[Ctrl-X] = $_pr_inline"));
	}

	#[test]
	fn converts_chained_lines_for_new_shells() {
		for shell in ["xonsh", "elvish"] {
			assert_eq!(shell_syntax(shell, "one &&\ntwo"), "one ;\ntwo");
		}
	}

	#[test]
	fn includes_privilege_in_history_command() {
		assert_eq!(
			history_command("bash", Some("sudo"), "echo 1"),
			"sudo echo 1"
		);
		assert_eq!(history_command("bash", None, "echo 1"), "echo 1");
		assert_eq!(
			history_command("bash", Some("sudo"), "echo 1 > /etc/systemfile"),
			"sudo bash -c \"echo 1 > /etc/systemfile\""
		);
	}
}
