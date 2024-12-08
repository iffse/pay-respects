use pay_respects_utils::evals::split_command;
use pay_respects_utils::files::get_path_files;
use std::process::exit;

use std::collections::HashMap;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

pub const PRIVILEGE_LIST: [&str; 2] = ["sudo", "doas"];

pub enum Mode {
	Suggestion,
	Cnf,
}
pub struct Init {
	pub shell: String,
	pub binary_path: String,
	pub alias: String,
	pub auto_alias: bool,
	pub cnf: bool,
}

impl Init {
	pub fn new() -> Init {
		Init {
			shell: String::from(""),
			binary_path: String::from(""),
			alias: String::from("f"),
			auto_alias: false,
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
		let (executables, modules, fallbacks) = {
			let path_executables = get_path_files();
			let mut executables = vec![];
			let mut modules = vec![];
			let mut fallbacks = vec![];
			for exe in path_executables {
				if exe.starts_with("pay-respects-module-") {
					modules.push(exe.to_string());
				} else if exe.starts_with("pay-respects-fallback-") {
					fallbacks.push(exe.to_string());
				} else {
					executables.push(exe.to_string());
				}
			}
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
	pub fn add_candidates(&mut self, candidates: &Vec<String>) {
		for candidate in candidates {
			let candidate = candidate.trim();
			if candidate != self.command {
				self.candidates.push(candidate.to_string());
			}
		}
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

pub fn get_error(shell: &str, command: &str) -> String {
	let error_msg = std::env::var("_PR_ERROR_MSG");
	let error = if let Ok(error_msg) = error_msg {
		std::env::remove_var("_PR_ERROR_MSG");
		error_msg
	} else {
		command_output_threaded(shell, command)
	};
	error.split_whitespace().collect::<Vec<&str>>().join(" ")
}

pub fn command_output_threaded(shell: &str, command: &str) -> String {
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

pub fn command_output(shell: &str, command: &str) -> String {
	let output = std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		.env("LC_ALL", "C")
		.output()
		.expect("failed to execute process");

	match output.stdout.is_empty() {
		false => String::from_utf8_lossy(&output.stdout).to_lowercase(),
		true => String::from_utf8_lossy(&output.stderr).to_lowercase(),
	}
}

pub fn module_output(data: &Data, module: &str) -> Option<Vec<String>> {
	let shell = &data.shell;
	let executable = &data.split[0];
	let last_command = &data.command;
	let error_msg = &data.error;
	let executables = data.executables.clone().join(",");
	let output = std::process::Command::new(shell)
		.arg("-c")
		.arg(module)
		.env("_PR_COMMAND", executable)
		.env("_PR_SHELL", shell)
		.env("_PR_LAST_COMMAND", last_command)
		.env("_PR_ERROR_MSG", error_msg)
		.env("_PR_EXECUTABLES", executables)
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
		Some(expanded)
	} else {
		None
	}
}

pub fn initialization(init: &mut Init) {
	let last_command;
	let shell_alias;
	let alias = &init.alias;
	let auto_alias = init.auto_alias;
	let cnf = init.cnf;
	let binary_path = &init.binary_path;

	match init.shell.as_str() {
		"bash" => {
			last_command = "$(history 2)";
			shell_alias = "$(alias)"
		}
		"zsh" => {
			last_command = "$(fc -ln -1)";
			shell_alias = "$(alias)"
		}
		"fish" => {
			last_command = "$(history | head -n 1)";
			shell_alias = "$(alias)";
		}
		"nu" | "nush" | "nushell" => {
			last_command = "(history | last).command";
			shell_alias = "\"\"";
			init.shell = "nu".to_string();
		}
		"pwsh" | "powershell" => {
			last_command = "Get-History | Select-Object -Last 1 | ForEach-Object {$_.CommandLine}";
			shell_alias = ";";
			init.shell = "pwsh".to_string();
		}
		_ => {
			println!("Unknown shell: {}", init.shell);
			return;
		}
	}

	let shell = &init.shell;

	if init.shell == "nu" {
		let init = format!(
			r#"
def --env {} [] {{
	let dir = (with-env {{ _PR_LAST_COMMAND: {}, _PR_SHELL: nu }} {{ {} }})
	cd $dir
}}
"#,
			init.alias, last_command, init.binary_path
		);
		println!("{}", init);
		return;
	}

	let mut initialize = match shell.as_str() {
		"bash" | "zsh" | "fish" => format!(
			"\
			eval $(_PR_LAST_COMMAND=\"{}\" \
			_PR_ALIAS=\"{}\" \
			_PR_SHELL=\"{}\" \
			\"{}\")",
			last_command, shell_alias, shell, binary_path
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
	if !auto_alias {
		println!("{}", initialize);
		return;
	}

	match shell.as_str() {
		"bash" | "zsh" => {
			initialize = format!(r#"alias {}='{}'"#, alias, initialize);
		}
		"fish" => {
			initialize = format!(
				r#"
function {} -d "Terminal command correction"
	eval $({})
end
"#,
				alias, initialize
			);
		}
		"pwsh" => {
			initialize = format!(
				"function {} {{\n{}",
				alias,
				initialize.split_once("\n").unwrap().1,
			);
		}
		_ => {
			println!("Unsupported shell: {}", shell);
			return;
		}
	}

	if cnf {
		match shell.as_str() {
			"bash" => {
				initialize = format!(
					r#"
command_not_found_handle() {{
	eval $(_PR_LAST_COMMAND="_ $@" _PR_SHELL="{}" _PR_MODE="cnf" "{}")
}}

{}
"#,
					shell, binary_path, initialize
				);
			}
			"zsh" => {
				initialize = format!(
					r#"
command_not_found_handler() {{
	eval $(_PR_LAST_COMMAND="$@" _PR_SHELL="{}" _PR_MODE="cnf" "{}")
}}

{}
"#,
					shell, binary_path, initialize
				);
			}
			"fish" => {
				initialize = format!(
					r#"
function fish_command_not_found --on-event fish_command_not_found
	eval $(_PR_LAST_COMMAND="$argv" _PR_SHELL="{}" _PR_MODE="cnf" "{}")
end

{}
"#,
					shell, binary_path, initialize
				);
			}
			"pwsh" => {
				initialize = format!(
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
					initialize, alias
				)
			}
			_ => {
				println!("Unsupported shell: {}", shell);
				return;
			}
		}
	}

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
