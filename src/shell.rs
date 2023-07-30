use std::{collections::HashMap, fs::read_to_string, process::exit};

pub fn find_shell() -> String {
	std::env::var("SHELL")
		.unwrap_or_else(|_| String::from("bash"))
		.rsplit('/')
		.next()
		.unwrap()
		.to_string()
		.to_lowercase()
}

pub fn find_last_command(shell: &str) -> String {
	let history_env = std::env::var("HISTFILE");
	let history_file = match history_env {
		Ok(file) => file,
		Err(_) => shell_default_history_file(shell),
	};

	let history = read_to_string(history_file).expect("Could not read history file.");

	match shell {
		"bash" => history.lines().rev().nth(1).unwrap().to_string(),
		"zsh" => history
			.lines()
			.rev()
			.nth(1)
			.unwrap()
			.split_once(';')
			.unwrap()
			.1
			.to_string(),
		"fish" => {
			let mut history_lines = history.lines().rev();
			let mut last_command = String::new();
			let mut skips = 0;
			while skips <= 2 {
				last_command = history_lines.next().unwrap().to_string();
				if last_command.starts_with("- cmd") {
					skips += 1;
				}
			}
			last_command.split_once(": ").unwrap().1.to_string()
		}
		_ => {
			println!("Unsupported shell.");
			exit(1);
		}
	}
}

pub fn command_output(shell: &str, command: &str) -> String {
	println!("Running command: {}", command);
	let output = std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		.stderr(std::process::Stdio::piped())
		.spawn()
		.expect("failed to execute process")
		.wait_with_output()
		.expect("failed to wait on process");

	String::from_utf8_lossy(&output.stderr)
		.to_string()
		.to_lowercase()
}

fn shell_default_history_file(shell: &str) -> String {
	let shell_file_map = HashMap::from([
		("bash", String::from(".bash_history")),
		("zsh", String::from(".zsh_history")),
		("fish", String::from(".local/share/fish/fish_history")),
	]);

	let file = shell_file_map.get(shell).expect("Unsupported shell.");
	format!("{}/{}", std::env::var("HOME").unwrap(), file)
}
