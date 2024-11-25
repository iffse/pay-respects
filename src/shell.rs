use std::process::exit;

use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

pub const PRIVILEGE_LIST: [&str; 2] = ["sudo", "doas"];

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
		"bash" => last_command,
		"zsh" => last_command,
		"fish" => last_command,
		"nu" => last_command,
		_ => last_command,
	}
}

pub fn expand_alias(shell: &str, full_command: &str) -> String {
	let alias_env = std::env::var("_PR_ALIAS");
	if alias_env.is_err() {
		return full_command.to_string();
	}
	let alias = alias_env.unwrap();
	if alias.is_empty() {
		return full_command.to_string();
	}

	let split_command = full_command.split_whitespace().collect::<Vec<&str>>();
	let (command, pure_command) = if PRIVILEGE_LIST.contains(&split_command[0]) {
		(split_command[1], Some(split_command[1..].join(" ")))
	} else {
		(split_command[0], None)
	};

	let mut expanded_command = Option::None;

	match shell {
		"bash" => {
			for line in alias.lines() {
				if line.starts_with(format!("alias {}=", command).as_str()) {
					let alias = line.replace(format!("alias {}='", command).as_str(), "");
					let alias = alias.trim_end_matches('\'').trim_start_matches('\'');

					expanded_command = Some(alias.to_string());
				}
			}
		}
		"zsh" => {
			for line in alias.lines() {
				if line.starts_with(format!("{}=", command).as_str()) {
					let alias = line.replace(format!("{}=", command).as_str(), "");
					let alias = alias.trim_start_matches('\'').trim_end_matches('\'');

					expanded_command = Some(alias.to_string());
				}
			}
		}
		"fish" => {
			for line in alias.lines() {
				if line.starts_with(format!("alias {} ", command).as_str()) {
					let alias = line.replace(format!("alias {} ", command).as_str(), "");
					let alias = alias.trim_start_matches('\'').trim_end_matches('\'');

					expanded_command = Some(alias.to_string());
				}
			}
		}
		_ => {
			eprintln!("Unsupported shell: {}", shell);
			exit(1);
		}
	};

	if expanded_command.is_none() {
		return full_command.to_string();
	};

	let expanded_command = expanded_command.unwrap();

	if pure_command.is_some() {
		let pure_command = pure_command.unwrap();
		if pure_command.starts_with(&expanded_command) {
			return full_command.to_string();
		}
	}

	full_command.replacen(command, &expanded_command, 1)
}

pub fn expand_alias_multiline(shell: &str, full_command: &str) -> String {
	let lines = full_command.lines().collect::<Vec<&str>>();
	let mut expanded = String::new();
	for line in lines {
		expanded = format!("{}\n{}", expanded, expand_alias(shell, line));
	}
	expanded
}

pub fn initialization(shell: &str, binary_path: &str, auto_alias: &str, cnf: bool) {
	let last_command;
	let alias;

	match shell {
		"bash" => {
			last_command = r"$(history 2 | head -n 1 | sed 's/^\s*[0-9]*//')";
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
			std::process::exit(1);
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
		std::process::exit(0);
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
    # restore mode from cnf
    if ($env:_PR_MODE -eq 'cnf') {{
        $env:_PR_MODE = $env:_PR_PWSH_ORIGIN_MODE;
        $env:_PR_PWSH_ORIGIN_MODE = $null;
    }}
}}
"#,
			last_command, shell, binary_path
		),
		_ => {
			println!("Unsupported shell: {}", shell);
			exit(1);
		}
	};
	if auto_alias.is_empty() {
		println!("{}", init);
		std::process::exit(0);
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
			exit(1);
		}
	}

	if cnf {
		match shell {
			"bash" | "zsh" => {
				init = format!(
					r#"
command_not_found_handler() {{
	eval $(_PR_LAST_COMMAND="$@" _PR_SHELL="{}" _PR_MODE=cnf "{}")
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
	eval $(_PR_LAST_COMMAND="$argv" _PR_SHELL="{}" _PR_MODE=cnf "{}")
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
				exit(1);
			}
		}
	}

	println!("{}", init);

	std::process::exit(0);
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
