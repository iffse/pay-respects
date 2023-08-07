use regex_lite::Regex;

use rule_parser::parse_rules;

use crate::files::{get_directory_files, get_path_files};
use crate::shell::{command_output, PRIVILEGE_LIST};
use crate::style::highlight_difference;

pub fn suggest_command(shell: &str, last_command: &str) -> Option<String> {
	let err = command_output(shell, last_command);

	let split_command = split_command(last_command);
	let executable = match PRIVILEGE_LIST.contains(&split_command[0].as_str()) {
		true => split_command.get(1).expect("No command found.").as_str(),
		false => split_command.first().expect("No command found.").as_str(),
	};

	if !PRIVILEGE_LIST.contains(&executable) {
		let suggest = match_pattern("privilege", last_command, &err, shell);
		if suggest.is_some() {
			return suggest;
		}
	}
	let suggest = match_pattern(executable, last_command, &err, shell);
	if let Some(suggest) = suggest {
		if PRIVILEGE_LIST.contains(&executable) {
			return Some(format!("{} {}", split_command[0], suggest));
		}
		return Some(suggest);
	}

	let suggest = match_pattern("general", last_command, &err, shell);
	if let Some(suggest) = suggest {
		return Some(suggest);
	}
	None
}

fn match_pattern(
	executable: &str,
	last_command: &str,
	error_msg: &str,
	shell: &str,
) -> Option<String> {
	parse_rules!("rules");
}

fn opt_regex(regex: &str, command: &mut String) -> String {
	let regex = Regex::new(regex).unwrap();
	let opts = regex
		.find_iter(command)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();
	for opt in opts.clone() {
		*command = command.replace(&opt, "");
	}
	opts.join(" ")
}

fn err_regex(regex: &str, error_msg: &str) -> String {
	let regex = Regex::new(regex).unwrap();
	let err = regex
		.find_iter(error_msg)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();

	err.join(" ")
}

fn eval_shell_command(shell: &str, command: &str) -> Vec<String> {
	let output = std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		.output()
		.expect("failed to execute process");
	let output = String::from_utf8_lossy(&output.stdout);
	let split_output = output.split('\n').collect::<Vec<&str>>();
	split_output
		.iter()
		.map(|s| s.trim().to_string())
		.collect::<Vec<String>>()
}

pub fn split_command(command: &str) -> Vec<String> {
	// this regex splits the command separated by spaces, except when the space
	// is escaped by a backslash or surrounded by quotes
	let regex = r#"([^\s"'\\]+|"(?:\\.|[^"\\])*"|\\.+|'(?:\\.|[^'\\])*'|\\.)+"#;
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
				if let Some(suggest) = find_similar(typo, path_files) {
					suggestion = suggest;
				}
			}
			"file" => {
				let files = get_directory_files(typo);
				if let Some(suggest) = find_similar(typo, files) {
					suggestion = suggest;
				}
			}
			_ => {}
		}
	} else if let Some(suggest) = find_similar(typo, candidates) {
		suggestion = suggest;
	}

	suggestion
}

fn find_similar(typo: &str, candidates: Vec<String>) -> Option<String> {
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

pub fn confirm_suggestion(shell: &str, command: &str, last_command: &str) {
	println!("{}\n", highlight_difference(shell, command, last_command));
	println!("Press enter to execute the suggestion. Or press Ctrl+C to exit.");
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
