// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::files::*;
use crate::settings::*;
use itertools::Itertools;
use regex_lite::Regex;

use std::collections::HashSet;
use std::cmp::Ordering;

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

pub fn regex_match(regex: &str, string: &str) -> bool {
	let regex = Regex::new(regex).unwrap();
	regex.is_match(string)
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

/// Returns the output of a shell command as a vector of strings
/// Each string is a line of output
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
		.filter(|s| !s.is_empty())
		.collect::<Vec<String>>()
}

/// Split the full command into command and arguments
pub fn split_command(command: &str) -> Vec<String> {
	#[cfg(debug_assertions)]
	eprintln!("command: {command}");
	// this regex splits the command separated by spaces, except when the space
	// - is escaped by a backslash
	// - surrounded by quotes
	// - is surrrounded by backticks
	let regex =
		r#"([^\s"'`\\]+|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|`(?:\\.|[^`\\])*`|\\ )+|\\+|\n"#;
	let regex = Regex::new(regex).unwrap();

	regex
		.find_iter(command)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>()
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
					if let Some(suggest) = find_similar(typo, executables) {
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
		} else if let Some(suggest) = find_similar(typo, candidates) {
			suggestions.push(suggest);
		} else {
			suggestions.push(typo.to_string());
		}
	}
	suggestions.join(" ")
}

pub fn best_match_path(typo: &str, executables: &[String]) -> Option<String> {
	find_similar(typo, executables)
}

pub fn best_matches_path(typo: &str, executables: &[String]) -> Option<Vec<String>> {
	find_similars(typo, executables)
}

pub fn get_initial_distance(str: &str) -> Option<usize> {
	let percentage = get_dl_distance_percentage();
	let threshold = get_dl_distance_threshold();
	let max = get_dl_distance_max();
	let min = get_dl_distance_min();

	if str.chars().count() < threshold {
		return None;
	}

	let distance = (str.chars().count() as f64 * percentage / 100.0).round() as usize;
	if distance < min {
		return None;
	}
	Some(std::cmp::min(distance + 1, max + 1))
}

pub fn find_similar(typo: &str, candidates: &[String]) -> Option<String> {
	let mut min_distance = get_initial_distance(typo)?;
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
		#[cfg(debug_assertions)]
		eprintln!("comparing '{typo}' with '{candidate}': distance = {distance}");

		// finding self, not a typo
		if distance == 0 {
			break;
		}
	}
	if let Some(min_distance_index) = min_distance_index {
		return Some(candidates[min_distance_index].to_string());
	}
	None
}

/// Similar to `find_similar`, but returns a vector of all candidates
/// with the same minimum distance
pub fn find_similars(typo: &str, candidates: &[String]) -> Option<Vec<String>> {
	let mut min_distance = get_initial_distance(typo)?;
	let mut min_distance_index = vec![];
	for (i, candidate) in candidates.iter().enumerate() {
		if candidate.is_empty() {
			continue;
		}
		let distance = compare_string(typo, candidate);
		#[cfg(debug_assertions)]
		eprintln!("comparing '{typo}' with '{candidate}': distance = {distance}");
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

/// Damerau-Levenshtein distance between two strings
#[allow(clippy::needless_range_loop)]
pub fn compare_string(a: &str, b: &str) -> usize {
	let a: Vec<char> = a.chars().collect();
	let b: Vec<char> = b.chars().collect();
	damerau_levenshtein_chars(&a, &b)
}

/// Trigram fuzzy similarity score between two strings in [0.0, 1.0]
pub fn trigram_fuzzy_score(query: &str, text: &str) -> f32 {
	if query.is_empty() || text.is_empty() {
		return 0.0;
	}

	let q = query.to_lowercase();
	let t = text.to_lowercase();

	// a matching substring should have a higher score
	let mut substring_bonus_score = 0.0;
	if t.contains(&q) {
		substring_bonus_score = 0.3;
	}

	let jaccard = trigram_jaccard_score(&q, &t);

	let score = jaccard * 0.7 + substring_bonus_score;

	score.min(1.0)
}


/// Result of a fuzzy search match
#[derive(Debug)]
pub struct Match<'a> {
	pub text: &'a str,
	pub score: f32,
}

/// Returns the top N candidates ranked by fuzzy similarity to the query
pub fn fuzzy_best_n(
	query: &str,
	candidates: &[&str],
	limit: usize,
) -> Vec<String> {
	let mut results: Vec<Match> = candidates
		.iter()
		.filter_map(|&candidate| {
			let score = trigram_edit_fuzzy_score(query, candidate);

			if score < 0.15 {
				None
			} else {
				Some(Match { text: candidate, score })
			}
		})
	.collect();

	results.sort_unstable_by(|a, b| {
		b.score
			.partial_cmp(&a.score)
			.unwrap_or(Ordering::Equal)
	});

	let best = results.first().map(|m| m.score).unwrap();
	// minimum tolerance allowed:
	let bottom_line = (best - 0.15).max(0.0);

	results.retain(|m| m.score >= bottom_line);
	results.truncate(limit);
	results.into_iter().map(|m| m.text.to_string()).collect()
}

/// A more sophisticated fuzzy score that combines trigram similarity, edit
/// distance, and bonuses for substring matches
pub fn trigram_edit_fuzzy_score(query: &str, text: &str) -> f32 {
	if query.is_empty() || text.is_empty() {
		return 0.0;
	}

	let query = query.to_lowercase();
	let text = text.to_lowercase();

	let tri = trigram_jaccard_score(&query, &text);
	// early rejection
	if tri < 0.1 {
		return tri;
	}

	// bonuses
	let mut bonus = 0.0;

	if text.contains(&query) {
		bonus += 0.4;
	}
	let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).collect();
	// word boundaries
	if words.iter()
		.any(|w| w.ends_with(&query)) {
		bonus += 0.2;
	}
	if words.iter()
		.any(|w| w.starts_with(&query))
	{
		bonus += 0.2;
	}

	let edit = best_substring_edit_score(&query, &text);

	(tri * 0.5 + edit * 0.4 + bonus).min(1.0)
}

fn trigrams(s: &str) -> HashSet<String> {
	let mut set = HashSet::new();
	let chars: Vec<char> = s.chars().collect();

	if chars.len() < 3 {
		set.insert(s.to_string());
		return set;
	}

	for i in 0..=chars.len() - 3 {
		let trigram: String = chars[i..i + 3].iter().collect();
		set.insert(trigram);
	}

	set
}

fn trigram_jaccard_score(a: &str, b: &str) -> f32 {
	let a_ngrams = trigrams(a);
	let b_ngrams = trigrams(b);

	let intersection = a_ngrams.intersection(&b_ngrams).count() as f32;
	let union = a_ngrams.union(&b_ngrams).count() as f32;

	if union == 0.0 {
		return 0.0;
	}

	intersection / union
}

/// Returns a similarity score in [0.0, 1.0]
/// by finding the best-matching substring of `text` against `query`.
pub fn best_substring_edit_score(query: &str, text: &str) -> f32 {
	let a: Vec<char> = query.chars().collect();
	let b: Vec<char> = text.chars().collect();

	let q_len = a.len();
	let t_len = b.len();

	if q_len == 0 || t_len == 0 {
		return 0.0;
	}

	// too short, no substring
	if t_len <= q_len {
		let dist = damerau_levenshtein_chars(&a, &b) as f32;
		return 1.0 - dist / q_len.max(t_len) as f32;
	}

	let mut best_norm = f32::MAX;

	// small variations
	let min_len = q_len.saturating_sub(2); // prevent underflow
	let max_len = q_len + 2;

	for start in 0..t_len {
		for len in min_len..=max_len {
			if start + len > t_len {
				continue;
			}

			let slice = &b[start..start + len];
			let dist = damerau_levenshtein_chars(&a, slice) as f32;
			let norm = dist / q_len.max(len) as f32;

			if norm < best_norm {
				best_norm = norm;

				// perfect match
				if best_norm == 0.0 {
					return 1.0;
				}
			}
		}
	}

	1.0 - best_norm
}

pub fn damerau_levenshtein_chars(a: &[char], b: &[char]) -> usize {
	let mut matrix = vec![vec![0; b.len() + 1]; a.len() + 1];

	for i in 0..=a.len() {
		matrix[i][0] = i;
	}
	for j in 0..=b.len() {
		matrix[0][j] = j;
	}

	for i in 1..=a.len() {
		for j in 1..=b.len() {
			let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };

			// deletion, insertion, substitution
			matrix[i][j] = (matrix[i - 1][j] + 1)
				.min(matrix[i][j - 1] + 1)
				.min(matrix[i - 1][j - 1] + cost);

			// transposition
			if i > 1 && j > 1
				&& a[i - 1] == b[j - 2]
					&& a[i - 2] == b[j - 1]
			{
				matrix[i][j] = matrix[i][j].min(matrix[i - 2][j - 2] + 1);
			}
		}
	}

	matrix[a.len()][b.len()]
}
