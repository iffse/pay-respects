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

pub fn initialization(shell: &str, binary_path: &str, auto_alias: &str) {
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

	let mut init = format!(
		"\
			eval $(_PR_LAST_COMMAND=\"{}\" \
			_PR_ALIAS=\"{}\" \
			_PR_SHELL=\"{}\" \
			\"{}\")",
		last_command, alias, shell, binary_path
	);

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
		_ => {
			println!("Unsupported shell: {}", shell);
			exit(1);
		}
	}

	println!("{}", init);

	std::process::exit(0);
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
