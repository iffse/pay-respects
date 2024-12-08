use regex_lite::Regex;
use crate::files::*;

pub fn opt_regex(regex: &str, command: &mut String) -> String {
	let regex = Regex::new(regex).unwrap();

	let mut opts = Vec::new();
	for captures in regex.captures_iter(command) {
		for cap in captures.iter().skip(1).flatten() {
			opts.push(cap.as_str().to_owned());
		}
	}

	for opt in opts.clone() {
		*command = command.replace(&opt, "");
	}
	opts.join(" ")
}

pub fn err_regex(regex: &str, error_msg: &str) -> String {
	let regex = Regex::new(regex).unwrap();

	let mut err = Vec::new();
	for captures in regex.captures_iter(error_msg) {
		for cap in captures.iter().skip(1).flatten() {
			err.push(cap.as_str().to_owned());
		}
	}
	err.join(" ")
}

pub fn cmd_regex(regex: &str, command: &str) -> String {
	let regex = Regex::new(regex).unwrap();

	let mut cmd = Vec::new();
	for captures in regex.captures_iter(command) {
		for cap in captures.iter().skip(1).flatten() {
			cmd.push(cap.as_str().to_owned());
		}
	}
	cmd.join(" ")
}

pub fn eval_shell_command(shell: &str, command: &str) -> Vec<String> {
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
	if cfg!(debug_assertions) {
		eprintln!("command: {command}")
	}
	// this regex splits the command separated by spaces, except when the space
	// is escaped by a backslash or surrounded by quotes
	let regex = r#"([^\s"'\\]+|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\\ )+|\\|\n"#;
	let regex = Regex::new(regex).unwrap();
	let split_command = regex
		.find_iter(command)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();
	split_command
}

pub fn suggest_typo(typos: &[String], candidates: Vec<String>, executables: &[String]) -> String {
	let mut suggestions = Vec::new();
	for typo in typos {
		let typo = typo.as_str();
		if candidates.len() == 1 {
			match candidates[0].as_str() {
				"path" => {
					if let Some(suggest) = find_similar(typo, executables, Some(2)) {
						suggestions.push(suggest);
					} else {
						suggestions.push(typo.to_string());
					}
				}
				"file" => {
					if let Some(suggest) = get_best_match_file(typo) {
						suggestions.push(suggest);
					} else {
						suggestions.push(typo.to_string());
					}
				}
				_ => {}
			}
		} else if let Some(suggest) = find_similar(typo, &candidates, Some(2)) {
			suggestions.push(suggest);
		} else {
			suggestions.push(typo.to_string());
		}
	}
	suggestions.join(" ")
}

pub fn best_match_path(typo: &str, executables: &[String]) -> Option<String> {
	find_similar(typo, executables, Some(3))
}

// higher the threshold, the stricter the comparison
// 1: anything
// 2: 50%
// 3: 33%
// ... etc
pub fn find_similar(typo: &str, candidates: &[String], threshold: Option<usize>) -> Option<String> {
	let threshold = threshold.unwrap_or(2);
	let mut min_distance = typo.chars().count() / threshold + 1;
	let mut min_distance_index = None;
	for (i, candidate) in candidates.iter().enumerate() {
		if candidate.is_empty() {
			continue;
		}
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
pub fn compare_string(a: &str, b: &str) -> usize {
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

