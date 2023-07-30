use std::collections::HashMap;

use rule_parser::parse_rules;

use crate::shell::{command_output, find_last_command, find_shell};
use crate::style::highlight_difference;

pub fn correct_command() -> Option<String> {
	let shell = find_shell();
	let last_command = find_last_command(&shell);
	let command_output = command_output(&shell, &last_command);
	println!("Last command: {}", last_command);
	println!("Command output: {}", command_output);

	let split_command = last_command.split_whitespace().collect::<Vec<&str>>();
	let command = match split_command.first().expect("No command found.") {
		&"sudo" => split_command.get(1).expect("No command found."),
		_ => split_command.first().expect("No command found."),
	};

	if split_command[0] != "sudo" {
		let suggest = match_pattern("sudo", &command_output);
		if let Some(suggest) = suggest {
			let suggest = eval_suggest(&suggest, &last_command);
			return Some(highlight_difference(&suggest, &last_command));
		}
	}

	let suggest = match_pattern(command, &command_output);
	if let Some(suggest) = suggest {
		let suggest = eval_suggest(&suggest, &last_command);
		return Some(highlight_difference(&suggest, &last_command));
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
			let start = start.parse::<usize>().unwrap();
			let end = end.parse::<usize>().unwrap();
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

pub fn confirm_correction(command: &str) {
	println!("Did you mean {}?", command);
	println!("Press enter to execute the corrected command. Or press Ctrl+C to exit.");
	std::io::stdin().read_line(&mut String::new()).unwrap();
	let shell = find_shell();
	std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		.spawn()
		.expect("failed to execute process");
}
