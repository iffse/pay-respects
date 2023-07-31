use std::process::exit;

pub const PRIVILEGE_LIST: [&str; 2] = ["sudo", "doas"];

pub fn command_output(shell: &str, command: &str) -> String {
	let output = std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		.output()
		.expect("failed to execute process");

	String::from_utf8_lossy(&output.stderr)
		.to_string()
		.split_whitespace()
		.collect::<Vec<&str>>()
		.join(" ")
		.to_lowercase()
}

fn last_command(shell: &str) -> String {
	let last_command = std::env::var("_PR_LAST_COMMAND").expect("No _PR_LAST_COMMAND in environment. Did you aliased the command with the correct argument?");
	match shell {
		"bash" => {
			let first_line = last_command.lines().next().unwrap();
			let split = first_line.split_whitespace().collect::<Vec<&str>>();
			split[1..].join(" ")
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
