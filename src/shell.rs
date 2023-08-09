use std::io::prelude::*;
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
		Ok(output) => String::from_utf8_lossy(&output.stderr)
			.to_string()
			.split_whitespace()
			.collect::<Vec<&str>>()
			.join(" ")
			.to_lowercase(),
		Err(_) => {
			use colored::*;
			eprintln!("Timeout while executing command: {}", command.red());
			exit(1);
		}
	}
}

fn last_command(shell: &str) -> String {
	let last_command = match std::env::var("_PR_LAST_COMMAND") {
		Ok(command) => command,
		Err(_) => {
			eprintln!("No _PR_LAST_COMMAND in environment. Did you aliased the command with the correct argument?\n\nUse `pay-respects -h` for help");
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
		_ => {
			eprintln!("Unsupported shell: {}", shell);
			exit(1);
		}
	}
}

pub fn last_command_expanded_alias(shell: &str) -> String {
	let alias = std::env::var("_PR_ALIAS").expect(
		"No _PR_ALIAS in environment. Did you aliased the command with the correct argument?",
	);
	let last_command = last_command(shell);
	if alias.is_empty() {
		return last_command;
	}

	let split_command = last_command.split_whitespace().collect::<Vec<&str>>();
	let command = if PRIVILEGE_LIST.contains(&split_command[0]) {
		split_command[1]
	} else {
		split_command[0]
	};

	let mut expanded_command = command.to_string();

	match shell {
		"bash" => {
			for line in alias.lines() {
				if line.starts_with(format!("alias {}=", command).as_str()) {
					let alias = line.replace(format!("alias {}='", command).as_str(), "");
					let alias = alias.trim_end_matches('\'').trim_start_matches('\'');

					expanded_command = alias.to_string();
				}
			}
		}
		"zsh" => {
			for line in alias.lines() {
				if line.starts_with(format!("{}=", command).as_str()) {
					let alias = line.replace(format!("{}=", command).as_str(), "");
					let alias = alias.trim_start_matches('\'').trim_end_matches('\'');

					expanded_command = alias.to_string();
				}
			}
		}
		"fish" => {
			for line in alias.lines() {
				if line.starts_with(format!("alias {} ", command).as_str()) {
					let alias = line.replace(format!("alias {} ", command).as_str(), "");
					let alias = alias.trim_start_matches('\'').trim_end_matches('\'');

					expanded_command = alias.to_string();
				}
			}
		}
		_ => {
			eprintln!("Unsupported shell: {}", shell);
			exit(1);
		}
	};

	last_command.replacen(command, &expanded_command, 1)
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
			let command = format!(
				"with-env {{ _PR_LAST_COMMAND : {},\
					_PR_ALIAS : {},\
					_PR_SHELL : nu }} \
					{{ {} }}",
				last_command, alias, binary_path
			);
			println!("{}\n", command);
			println!("Add following to your config file? (Y/n)");
			let alias = format!("alias f = {}", command);
			println!("{}", alias);
			let mut input = String::new();
			std::io::stdin().read_line(&mut input).unwrap();
			match input.trim() {
				"Y" | "y" | "" => {
					let output = std::process::Command::new("nu")
						.arg("-c")
						.arg("echo $nu.config-path")
						.output()
						.expect("Failed to execute process");
					let config_path = String::from_utf8_lossy(&output.stdout);
					let mut file = std::fs::OpenOptions::new()
						.write(true)
						.append(true)
						.open(config_path.trim())
						.expect("Failed to open config file");

					writeln!(file, "{}", alias).expect("Failed to write to config file");
				}
				_ => std::process::exit(0),
			};
			std::process::exit(0);
		}
		_ => {
			println!("Unknown shell: {}", shell);
			std::process::exit(1);
		}
	}

	let mut init = format!(
		"\
			_PR_LAST_COMMAND=\"{}\" \
			_PR_ALIAS=\"{}\" \
			_PR_SHELL=\"{}\" \
			\"{}\"",
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
	{}
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
