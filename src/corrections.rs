use std::collections::HashMap;

use rule_parser::parse_rules;

use crate::shell::command_output;
use crate::style::highlight_difference;

pub fn correct_command(shell: &str, last_command: &str) -> Option<String> {
	let command_output = command_output(shell, last_command);

	let split_command = last_command.split_whitespace().collect::<Vec<&str>>();
	let command = match split_command.first().expect("No command found.") {
		&"sudo" => split_command.get(1).expect("No command found."),
		_ => split_command.first().expect("No command found."),
	};

	if split_command[0] != "sudo" {
		let suggest = match_pattern("sudo", &command_output);
		if let Some(suggest) = suggest {
			let suggest = eval_suggest(&suggest, last_command);
			return Some(suggest);
		}
	}
	let suggest = match_pattern(command, &command_output);
	if let Some(suggest) = suggest {
		let suggest = eval_suggest(&suggest, last_command);
		if split_command[0] == "sudo" {
			return Some(format!("sudo {}", suggest));
		}
		return Some(suggest);
	}
	None
}

fn match_pattern(command: &str, error_msg: &str) -> Option<String> {
	let rules = parse_rules!("rules");
	if rules.contains_key(command) {
		let suggest = rules.get(command).unwrap();
		for (pattern, suggest) in suggest {
			for pattern in pattern {
				if error_msg.contains(pattern) {
					return Some(suggest.to_owned().to_string());
				}
			}
		}
		None
	} else {
		None
	}
}

fn eval_suggest(suggest: &str, last_command: &str) -> String {
	let mut suggest = suggest.to_owned();
	if suggest.contains("{{command}}") {
		suggest = suggest.replace("{{command}}", last_command);
	}
	while suggest.contains("{{command") {
		let placeholder_start = "{{command";
		let placeholder_end = "}}";
		let placeholder = suggest.find(placeholder_start).unwrap()
			..suggest.find(placeholder_end).unwrap() + placeholder_end.len();

		let range = suggest[placeholder.to_owned()].trim_matches(|c| c == '[' || c == ']');
		if let Some((start, end)) = range.split_once(':') {
			let start = match start {
				"" => 0,
				_ => start.parse::<usize>().unwrap(),
			};
			let end = match end {
				"" => last_command.split_whitespace().count(),
				_ => end.parse::<usize>().unwrap(),
			};
			let split_command = last_command.split_whitespace().collect::<Vec<&str>>();
			let command = split_command[start..end].join(" ");
			suggest = suggest.replace(&suggest[placeholder], &command);
		} else {
			let range = range.parse::<usize>().unwrap();
			let split_command = last_command.split_whitespace().collect::<Vec<&str>>();
			let command = split_command[range].to_owned();
			suggest = suggest.replace(&suggest[placeholder], &command);
		}
	}

	suggest
}

pub fn confirm_correction(shell: &str, command: &str, last_command: &str) {
	println!(
		"Did you mean {}?",
		highlight_difference(command, last_command)
	);
	println!("Press enter to execute the corrected command. Or press Ctrl+C to exit.");
	std::io::stdin().read_line(&mut String::new()).unwrap();

	if command.starts_with("sudo") {
		std::process::Command::new("sudo")
			.arg(shell)
			.arg("-c")
			.arg(command)
			.spawn()
			.expect("failed to execute process")
			.wait()
			.expect("failed to wait on process");
	} else {
		std::process::Command::new(shell)
			.arg("-c")
			.arg(command)
			.spawn()
			.expect("failed to execute process")
			.wait()
			.expect("failed to wait on process");
	}
}
