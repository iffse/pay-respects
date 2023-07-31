use std::collections::HashMap;

use rule_parser::parse_rules;

use crate::shell::{command_output, PRIVILEGE_LIST};
use crate::style::highlight_difference;

pub fn correct_command(shell: &str, last_command: &str) -> Option<String> {
	let err = command_output(shell, last_command);

	let split_command = last_command.split_whitespace().collect::<Vec<&str>>();
	let executable = match PRIVILEGE_LIST.contains(&split_command[0]) {
		true => split_command.get(1).expect("No command found."),
		false => split_command.first().expect("No command found."),
	};

	if !PRIVILEGE_LIST.contains(executable) {
		let suggest = match_pattern("privilege", last_command, &err);
		if let Some(suggest) = suggest {
			let suggest = eval_suggest(&suggest, last_command);
			return Some(suggest);
		}
	}
	let suggest = match_pattern(executable, last_command, &err);
	if let Some(suggest) = suggest {
		let suggest = eval_suggest(&suggest, last_command);
		if PRIVILEGE_LIST.contains(executable) {
			return Some(format!("{} {}", split_command[0], suggest));
		}
		return Some(suggest);
	}
	None
}

fn match_pattern(executable: &str, command: &str, error_msg: &str) -> Option<String> {
	let rules = parse_rules!("rules");
	if rules.contains_key(executable) {
		let suggest = rules.get(executable).unwrap();
		for (pattern, suggest) in suggest {
			for pattern in pattern {
				if error_msg.contains(pattern) {
					for suggest in suggest {
						if let Some(suggest) = check_suggest(suggest, command, error_msg) {
							return Some(suggest);
						}
					}
				}
			}
		}
		None
	} else {
		None
	}
}

fn check_suggest(suggest: &str, command: &str, error_msg: &str) -> Option<String> {
	if !suggest.starts_with('#') {
		return Some(suggest.to_owned());
	}
	let lines = suggest.lines().collect::<Vec<&str>>();
	let conditions = lines.first().unwrap().trim().replacen('#', "", 1);
	let conditions = conditions.trim_start_matches('[').trim_end_matches(']');
	let conditions = conditions.split(',').collect::<Vec<&str>>();

	for condition in conditions {
		let (mut condition, arg) = condition.split_once('(').unwrap();
		condition = condition.trim();
		let arg = arg.trim_start_matches('(').trim_end_matches(')');
		let reverse = match condition.starts_with('!') {
			true => {
				condition = condition.trim_start_matches('!');
				true
			}
			false => false,
		};
		if eval_condition(condition, arg, command, error_msg) == reverse {
			return None;
		}
	}
	Some(lines[1..].join("\n"))
}

fn eval_condition(condition: &str, arg: &str, command: &str, error_msg: &str) -> bool {
	match condition {
		"executable" => {
			let output = std::process::Command::new("which")
				.arg(arg)
				.output()
				.expect("failed to execute process");
			output.status.success()
		}
		"err_contains" => error_msg.contains(arg),
		"cmd_contains" => command.contains(arg),
		_ => unreachable!("Unknown condition when evaluation condition: {}", condition),
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

	for p in PRIVILEGE_LIST {
		let _p = p.to_owned() + " ";
		if command.starts_with(&_p) {
			let command = command.replace(p, "");
			std::process::Command::new(p)
				.arg(shell)
				.arg("-c")
				.arg(command)
				.spawn()
				.expect("failed to execute process")
				.wait()
				.expect("failed to wait on process");
			return;
		}
	}

	std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		.spawn()
		.expect("failed to execute process")
		.wait()
		.expect("failed to wait on process");
}
