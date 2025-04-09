// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::files::*;
use itertools::Itertools;
use regex_lite::Regex;

fn regex_captures(regex: &str, string: &str) -> Vec<String> {
	let regex = Regex::new(regex).unwrap();

	let mut caps = Vec::new();
	for captures in regex.captures_iter(string) {
		for cap in captures.iter().skip(1).flatten() {
			caps.push(cap.as_str().to_owned());
		}
	}
	caps
}

pub fn opt_regex(regex: &str, command: &mut String) -> String {
	let opts = regex_captures(regex, command);

	for opt in opts.clone() {
		*command = command.replace(&opt, "");
	}
	opts.join(" ")
}

pub fn err_regex(regex: &str, error_msg: &str) -> String {
	let err = regex_captures(regex, error_msg);
	err.join(" ")
}

pub fn cmd_regex(regex: &str, command: &str) -> String {
	let cmd = regex_captures(regex, command);
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
	#[cfg(debug_assertions)]
	eprintln!("command: {command}");
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

pub fn suggest_typo(typos: &[String], candidates: &[String], executables: &[String]) -> String {
	let mut suggestions = Vec::new();
	for typo in typos {
		let typo = typo.as_str();
		if candidates.len() == 1 {
			match candidates[0].as_str() {
				"path" => {
					if typo.contains(std::path::MAIN_SEPARATOR) {
						if let Some(suggest) = best_match_file(typo) {
							suggestions.push(suggest);
						} else {
							suggestions.push(typo.to_string());
						}
						continue;
					}
					if let Some(suggest) = find_similar(typo, executables, Some(2)) {
						suggestions.push(suggest);
					} else {
						suggestions.push(typo.to_string());
					}
				}
				"file" => {
					if let Some(suggest) = best_match_file(typo) {
						suggestions.push(suggest);
					} else {
						suggestions.push(typo.to_string());
					}
				}
				_ => {
					unreachable!("suggest_typo: must have at least two candidates")
				}
			}
		} else if let Some(suggest) = find_similar(typo, candidates, Some(2)) {
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

pub fn best_matches_path(typo: &str, executables: &[String]) -> Option<Vec<String>> {
	find_similars(typo, executables, Some(3))
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

pub fn find_similars(
	typo: &str,
	candidates: &[String],
	threshold: Option<usize>,
) -> Option<Vec<String>> {
	let threshold = threshold.unwrap_or(2);
	let mut min_distance = typo.chars().count() / threshold + 1;
	let mut min_distance_index = vec![];
	for (i, candidate) in candidates.iter().enumerate() {
		if candidate.is_empty() {
			continue;
		}
		let distance = compare_string(typo, candidate);
		use std::cmp::Ordering::*;
		match distance.cmp(&min_distance) {
			Equal => {
				if !min_distance_index.is_empty() {
					min_distance_index.push(i)
				}
			}
			Less => {
				min_distance = distance;
				min_distance_index.clear();
				min_distance_index.push(i);
			}
			_ => {}
		}
	}
	if !min_distance_index.is_empty() {
		return Some(
			min_distance_index
				.iter()
				.map(|&i| candidates[i].to_string())
				.collect::<Vec<String>>()
				.into_iter()
				.unique()
				.collect(),
		);
	}
	None
}

/// Damerau-Levenshtein distance algorithm
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

			// addition for optimal string alignment distance
			if i > 0
				&& j > 0 && ca == b.chars().nth(j - 1).unwrap()
				&& a.chars().nth(i - 1).unwrap() == cb
			{
				matrix[i + 1][j + 1] =
					std::cmp::min(matrix[i + 1][j + 1], matrix[i - 1][j - 1] + 1);
			}
		}
	}
	matrix[a.chars().count()][b.chars().count()]
}
