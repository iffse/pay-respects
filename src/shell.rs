use std::process::exit;

use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use regex_lite::Regex;

pub const PRIVILEGE_LIST: [&str; 2] = ["sudo", "doas"];

pub enum Mode {
	Suggestion,
	Cnf,
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
	pub mode: Mode,
}

pub struct Init {
	pub shell: String,
	pub binary_path: String,
	pub auto_alias: String,
	pub cnf: bool,
}

impl Data {
	pub fn init() -> Data {
		let shell = get_shell();
		let command = last_command(&shell).trim().to_string();
		let alias = alias_map(&shell);
		let mode = run_mode();

		let mut init = Data {
			shell,
			command,
			suggest: None,
			candidates: vec![],
			alias,
			split: vec![],
			privilege: None,
			error: "".to_string(),
			mode,
		};

		init.split();
		init.update_error(None);
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
			self.command = self.command.replacen(&split[0], "", 1);
			self.privilege = Some(split.remove(0))
		}
		self.split = split;
	}

	pub fn update_error(&mut self, error: Option<String>) {
		if let Some(error) = error {
			self.error = error;
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

	pub fn add_candidate(&mut self, candidate: &str) {
		let candidate = candidate.trim();
		if candidate != self.command {
			self.candidates.push(candidate.to_string());
		}
	}
}

pub fn split_command(command: &str) -> Vec<String> {
	#[cfg(debug_assertions)]
	eprintln!("command: {command}");
	// this regex splits the command separated by spaces, except when the space
	// is escaped by a backslash or surrounded by quotes
	let regex = r#"([^\s"'\\]+|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\\ )+|\\|\n"#;
	let regex = Regex::new(regex).unwrap();
	let split_command = regex
		.find_iter(command)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();
	#[cfg(debug_assertions)]
	eprintln!("split_command: {:?}", split_command);
	split_command
}

pub fn get_error(shell: &str, command: &str) -> String {
	let error_msg = std::env::var("_PR_ERROR_MSG");
	let error = if let Ok(error_msg) = error_msg {
		std::env::remove_var("_PR_ERROR_MSG");
		error_msg
	} else {
		command_output(shell, command)
	};
	error.split_whitespace().collect::<Vec<&str>>().join(" ")
}

pub fn command_output(shell: &str, command: &str) -> String {
	let (sender, receiver) = channel();

	let _shell = shell.to_owned();
	let _command = command.to_owned();
	thread::spawn(move || {
		sender
			.send(
				std::process::Command::new(_shell)
					.arg("-c")
					.arg(_command)
					.env("LC_ALL", "C")
					.output()
					.expect("failed to execute process"),
			)
			.expect("failed to send output");
	});

	match receiver.recv_timeout(Duration::from_secs(3)) {
		Ok(output) => match output.stderr.is_empty() {
			true => String::from_utf8_lossy(&output.stdout).to_lowercase(),
			false => String::from_utf8_lossy(&output.stderr).to_lowercase(),
		},
		Err(_) => {
			use colored::*;
			eprintln!("Timeout while executing command: {}", command.red());
			exit(1);
		}
	}
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
		"bash" => {
			let first_line = last_command.lines().next().unwrap().trim();
			first_line.split_once(' ').unwrap().1.to_string()
		}
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
			_ => Mode::Suggestion,
		},
		Err(_) => Mode::Suggestion,
	}
}

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
		_ => {
			unreachable!("Unsupported shell: {}", shell);
		}
	}
	Some(alias_map)
}

pub fn expand_alias(map: &HashMap<String, String>, command: &str) -> Option<String> {
	let (command, args) = if let Some(split) = command.split_once(' ') {
		(split.0, split.1)
	} else {
		(command, "")
	};
	if let Some(expand) = map.get(command) {
		Some(format!("{} {}", expand, args))
	} else {
		None
	}
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
		Some(expanded)
	} else {
		None
	}
}

pub fn initialization(shell: &str, binary_path: &str, auto_alias: &str, cnf: bool) {
	let last_command;
	let alias;

	match shell {
		"bash" => {
			last_command = "$(history 2)";
			alias = "$(alias)"
		}
		"zsh" => {
			last_command = "$(fc -ln -1)";
			alias = "$(alias)"
		}
		"fish" => {
			last_command = "$(history | head -n 1)";
			alias = "$(alias)";
		}
		"nu" | "nush" | "nushell" => {
			last_command = "(history | last).command";
			alias = "\"\"";
		}
		"pwsh" | "powershell" => {
			last_command = "Get-History | Select-Object -Last 1 | ForEach-Object {$_.CommandLine}";
			alias = ";";
		}
		_ => {
			println!("Unknown shell: {}", shell);
			return;
		}
	}

	if shell == "nu" || shell == "nush" || shell == "nushell" {
		let pr_alias = if auto_alias.is_empty() {
			"f"
		} else {
			auto_alias
		};

		let init = format!(
			r#"
def --env {} [] {{
	let dir = (with-env {{ _PR_LAST_COMMAND: {}, _PR_SHELL: nu }} {{ {} }})
	cd $dir
}}
"#,
			pr_alias, last_command, binary_path
		);
		println!("{}", init);
		return;
	}

	let mut init = match shell {
		"bash" | "zsh" | "fish" => format!(
			"\
			eval $(_PR_LAST_COMMAND=\"{}\" \
			_PR_ALIAS=\"{}\" \
			_PR_SHELL=\"{}\" \
			\"{}\")",
			last_command, alias, shell, binary_path
		),
		"pwsh" | "powershell" => format!(
			r#"& {{
	try {{
		# fetch command and error from session history only when not in cnf mode
		if ($env:_PR_MODE -ne 'cnf') {{
			$env:_PR_LAST_COMMAND = ({});
			$err = Get-Error;
			if ($env:_PR_LAST_COMMAND -eq $err.InvocationInfo.Line) {{
				$env:_PR_ERROR_MSG = $err.Exception.Message
			}}
		}}
		$env:_PR_SHELL = '{}';
		&'{}';
	}}
	finally {{
		# restore mode from cnf
		if ($env:_PR_MODE -eq 'cnf') {{
			$env:_PR_MODE = $env:_PR_PWSH_ORIGIN_MODE;
			$env:_PR_PWSH_ORIGIN_MODE = $null;
		}}
	}}
}}
"#,
			last_command, shell, binary_path
		),
		_ => {
			println!("Unsupported shell: {}", shell);
			return;
		}
	};
	if auto_alias.is_empty() {
		println!("{}", init);
		return;
	}

	match shell {
		"bash" | "zsh" => {
			init = format!(r#"alias {}='{}'"#, auto_alias, init);
		}
		"fish" => {
			init = format!(
				r#"
function {} -d "Terminal command correction"
	eval $({})
end
"#,
				auto_alias, init
			);
		}
		"pwsh" | "powershell" => {
			init = format!(
				"function {} {{\n{}",
				auto_alias,
				init.split_once("\n").unwrap().1,
			);
		}
		_ => {
			println!("Unsupported shell: {}", shell);
			return;
		}
	}

	if cnf {
		match shell {
			"bash" => {
				init = format!(
					r#"
command_not_found_handle() {{
	eval $(_PR_LAST_COMMAND="_ $@" _PR_SHELL="{}" _PR_MODE="cnf" "{}")
}}

{}
"#,
					shell, binary_path, init
				);
			}
			"zsh" => {
				init = format!(
					r#"
command_not_found_handler() {{
	eval $(_PR_LAST_COMMAND="$@" _PR_SHELL="{}" _PR_MODE="cnf" "{}")
}}

{}
"#,
					shell, binary_path, init
				);
			}
			"fish" => {
				init = format!(
					r#"
function fish_command_not_found --on-event fish_command_not_found
	eval $(_PR_LAST_COMMAND="$argv" _PR_SHELL="{}" _PR_MODE="cnf" "{}")
end

{}
"#,
					shell, binary_path, init
				);
			}
			"pwsh" | "powershell" => {
				init = format!(
					r#"{}
$ExecutionContext.InvokeCommand.CommandNotFoundAction =
{{
	param(
		[string]
		$commandName,
		[System.Management.Automation.CommandLookupEventArgs]
		$eventArgs
	)
	# powershell does not support run command with specific environment variables
	# but you must set global variables. so we are memorizing the current mode and the alias function will reset it later.
	$env:_PR_PWSH_ORIGIN_MODE=$env:_PR_MODE;
	$env:_PR_MODE='cnf';
	# powershell may search command with prefix 'get-' or '.\' first when this hook is hit, strip them
	$env:_PR_LAST_COMMAND=$commandName -replace '^get-|\.\\','';
	$eventArgs.Command = (Get-Command {});
	$eventArgs.StopSearch = $True;
}}
"#,
					init, auto_alias
				)
			}
			_ => {
				println!("Unsupported shell: {}", shell);
				return;
			}
		}
	}

	println!("{}", init);
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

pub fn shell_syntax(shell: &str, command: &mut String) {
	#[allow(clippy::single_match)]
	match shell {
		"nu" => {
			*command = command.replace(" && ", " and ");
		}
		_ => {}
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
