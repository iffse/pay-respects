use std::collections::HashMap;

use rule_parser::parse_rules;

use crate::shell::{command_output, PRIVILEGE_LIST};
use crate::style::highlight_difference;

pub fn correct_command(shell: &str, last_command: &str) -> Option<String> {
	let err = command_output(shell, last_command);

	let split_command = split_command(last_command);
	let executable = match PRIVILEGE_LIST.contains(&split_command[0].as_str()) {
		true => split_command.get(1).expect("No command found.").as_str(),
		false => split_command.first().expect("No command found.").as_str(),
	};

	if !PRIVILEGE_LIST.contains(&executable) {
		let suggest = match_pattern("privilege", last_command, &err);
		if let Some(suggest) = suggest {
			let suggest = eval_suggest(&suggest, last_command);
			return Some(suggest);
		}
	}
	let suggest = match_pattern(executable, last_command, &err);
	if let Some(suggest) = suggest {
		let suggest = eval_suggest(&suggest, last_command);
		if PRIVILEGE_LIST.contains(&executable) {
			return Some(format!("{} {}", split_command[0], suggest));
		}
		return Some(suggest);
	}

	let suggest = match_pattern("no_command", last_command, &err);
	if let Some(suggest) = suggest {
		let suggest = eval_suggest(&suggest, last_command);
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
						if let Some(suggest) = check_condition(suggest, command, error_msg) {
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

fn check_condition(suggest: &str, command: &str, error_msg: &str) -> Option<String> {
	if !suggest.starts_with('#') {
		return Some(suggest.to_owned());
	}
	let mut lines = suggest.lines().collect::<Vec<&str>>();
	let mut conditions = String::new();
	for (i, line) in lines[0..].iter().enumerate() {
		conditions.push_str(line);
		if line.ends_with(']') {
			lines = lines[i..].to_vec();
			break;
		}
	}
	let conditions = conditions
		.trim_start_matches(['#', '['])
		.trim_end_matches(']')
		.split(',')
		.collect::<Vec<&str>>();

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
		"match_typo_command" => false,
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

		let start_index = suggest.find(placeholder_start).unwrap();
		let end_index = suggest[start_index..].find(placeholder_end).unwrap()
			+ start_index
			+ placeholder_end.len();

		let placeholder = start_index + placeholder_start.len()..end_index - placeholder_end.len();
		let range = suggest[placeholder.to_owned()].trim_matches(|c| c == '[' || c == ']');
		if let Some((start, end)) = range.split_once(':') {
			let split_command = split_command(last_command);
			let start = {
				let mut start = start.parse::<i32>().unwrap_or(0);
				if start < 0 {
					start += split_command.len() as i32;
				}
				start as usize
			};
			let end = {
				let mut end = end.parse::<i32>().unwrap_or(split_command.len() as i32 - 1) + 1;
				if end < 0 {
					end += split_command.len() as i32;
				}
				end as usize
			};
			let command = split_command[start..end].join(" ");

			suggest = suggest.replace(&suggest[start_index..end_index], &command);
		} else {
			let range = range.parse::<usize>().unwrap_or(0);
			let split_command = split_command(last_command);
			let command = split_command[range].to_owned();
			suggest = suggest.replace(&suggest[start_index..end_index], &command);
		}
	}

	while suggest.contains("{{typo") {
		let placeholder_start = "{{typo";
		let placeholder_end = "}}";

		let start_index = suggest.find(placeholder_start).unwrap();
		let end_index = suggest[start_index..].find(placeholder_end).unwrap()
			+ start_index
			+ placeholder_end.len();

		let placeholder = start_index + placeholder_start.len()..end_index - placeholder_end.len();

		let mut command_index;
		let mut match_list = vec![];
		if suggest.contains('[') {
			let split = suggest[placeholder.to_owned()]
				.split(&['[', ']'])
				.collect::<Vec<&str>>();
			command_index = split[1].parse::<i32>().unwrap();
			if command_index < 0 {
				command_index += split_command(last_command).len() as i32;
			}
		} else {
			unreachable!("Typo suggestion must have a command index");
		}
		if suggest.contains('(') {
			let split = suggest[placeholder.to_owned()]
				.split(&['(', ')'])
				.collect::<Vec<&str>>();
			match_list = split[1].split(',').collect::<Vec<&str>>();
		}

		let command = split_command(last_command);
		let match_list = match_list
			.iter()
			.map(|s| s.to_string())
			.collect::<Vec<String>>();
		let command_index = command_index as usize;
		let suggestion = suggest_typo(&command[command_index], match_list.clone());

		suggest = suggest.replace(&suggest[start_index..end_index], &suggestion);
	}

	suggest
}

pub fn split_command(command: &str) -> Vec<String> {
	use regex::Regex;
	let regex = r#"([^\s"\\]+|"(?:\\.|[^"\\])*"|\\.)+"#;
	let regex = Regex::new(regex).unwrap();
	let split_command = regex
		.find_iter(command)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();
	split_command
}

fn suggest_typo(typo: &str, candidates: Vec<String>) -> String {
	let mut suggestion = typo.to_owned();

	if candidates.len() == 1 {
		match candidates[0].as_str() {
			"path" => {
				let path_files = get_path_files();
				if let Some(suggest) = find_fimilar(typo, path_files) {
					suggestion = suggest;
				}
			}
			"file" => {
				let files = get_directory_files(typo);
				if let Some(suggest) = find_fimilar(typo, files) {
					suggestion = suggest;
				}
			}
			_ => {}
		}
	} else if let Some(suggest) = find_fimilar(typo, candidates) {
		suggestion = suggest;
	}

	suggestion
}

fn get_path_files() -> Vec<String> {
	let path = std::env::var("PATH").unwrap();
	let path = path.split(':').collect::<Vec<&str>>();
	let mut all_executable = vec![];
	for p in path {
		let files = match std::fs::read_dir(p) {
			Ok(files) => files,
			Err(_) => continue,
		};
		for file in files {
			let file = file.unwrap();
			let file_name = file.file_name().into_string().unwrap();
			all_executable.push(file_name);
		}
	}
	all_executable
}

fn get_directory_files(input: &str) -> Vec<String> {
	let mut input = input.trim_matches(|c| c == '\'' || c == '"').to_owned();
	let files = loop {
		match std::fs::read_dir(&input) {
			Ok(files) => break files,
			Err(_) => {
				if let Some((dirs, _)) = input.rsplit_once('/') {
					input = dirs.to_owned();
				} else {
					break std::fs::read_dir("./").unwrap()
				}
			}
		}
	};

	let mut all_files = vec![];
	for file in files {
		let file = file.unwrap();
		let file_name = file.path().to_str().unwrap().to_owned();
		all_files.push(file_name);
	}
	all_files
}

fn find_fimilar(typo: &str, candidates: Vec<String>) -> Option<String> {
	let mut min_distance = 10;
	let mut min_distance_index = None;
	for (i, candidate) in candidates.iter().enumerate() {
		let distance = compare_string(typo, candidate);
		if distance < min_distance {
			min_distance = distance;
			min_distance_index = Some(i);
		}
	}
	if let Some(min_distance_index) = min_distance_index {
		return Some(candidates[min_distance_index].to_string());
	}
	None
}

// warning disable
#[allow(clippy::needless_range_loop)]
fn compare_string(a: &str, b: &str) -> usize {
	let mut matrix = vec![vec![0; b.chars().count() + 1]; a.chars().count() + 1];

	for i in 0..a.chars().count() + 1 {
		matrix[i][0] = i;
	}
	for j in 0..b.chars().count() + 1 {
		matrix[0][j] = j;
	}

	for (i, ca) in a.chars().enumerate() {
		for (j, cb) in b.chars().enumerate() {
			let cost = if ca == cb { 0 } else { 1 };
			matrix[i + 1][j + 1] = std::cmp::min(
				std::cmp::min(matrix[i][j + 1] + 1, matrix[i + 1][j] + 1),
				matrix[i][j] + cost,
			);
		}
	}
	matrix[a.chars().count()][b.chars().count()]
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
