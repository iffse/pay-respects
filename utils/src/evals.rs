// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::files::*;
use crate::settings::*;
use itertools::Itertools;
use regex_lite::Regex;

use std::cmp::Ordering;
use std::collections::HashSet;

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

pub fn split_comment(split: &mut Vec<String>) -> Option<String> {
	let mut comments: Option<String> = None;
	if split.contains(&"#".to_string()) {
		// remove everything after the first # and store it in comments
		let index = split.iter().position(|s| s == "#").unwrap();
		let comments_vec = split.split_off(index);
		comments = Some(comments_vec.join(" "));
		if split.is_empty() {
			*split = vec!["".to_string()];
		}
	}
	comments
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

pub fn best_match(query: &str, dict: &[String]) -> Option<String> {
	find_similar(query, dict)
}

pub fn best_matches(query: &str, dict: &[String]) -> Option<Vec<String>> {
	find_similars(query, dict)
}

pub fn get_initial_distance(str: &str) -> Option<usize> {
	let percentage = get_dl_distance_percentage();
	let threshold = get_search_threshold();
	let max = get_dl_distance_max();
	let min = get_dl_distance_min();

	if str.chars().count() < threshold {
		return None;
	}

	let distance = (str.chars().count() as f32 * percentage / 100.0).round() as usize;
	if distance < min {
		return None;
	}
	Some(std::cmp::min(distance + 1, max + 1))
}

pub fn find_similar(typo: &str, candidates: &[String]) -> Option<String> {
	if typo.len() < get_search_threshold() {
		return None;
	}
	match get_search_type() {
		SearchType::DamerauLevenshtein => edit_distance_best(typo, candidates),
		SearchType::TrigramDamerauLevenshtein => {
			fuzzy_best(typo, candidates, get_trigram_minimum_score())
		}
	}
}

pub fn find_similars(typo: &str, candidates: &[String]) -> Option<Vec<String>> {
	if typo.len() < get_search_threshold() {
		return None;
	}
	match get_search_type() {
		SearchType::DamerauLevenshtein => edit_distance_bests(typo, candidates),
		SearchType::TrigramDamerauLevenshtein => {
			fuzzy_best_n(typo, candidates, get_trigram_minimum_score(), 9)
		}
	}
}

pub fn edit_distance_best(typo: &str, candidates: &[String]) -> Option<String> {
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

pub fn edit_distance_bests(typo: &str, candidates: &[String]) -> Option<Vec<String>> {
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

pub fn fuzzy_best(query: &str, candidates: &[String], minimum_score: f32) -> Option<String> {
	let results = fuzzy_best_n(query, candidates, minimum_score, 1);
	if let Some(mut results) = results {
		results.pop()
	} else {
		None
	}
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
	candidates: &[String],
	minimum_score: f32,
	limit: usize,
) -> Option<Vec<String>> {
	fuzzy_best_n_main(query, candidates, minimum_score, limit, false)
}

pub fn fuzzy_best_n_substring(
	query: &str,
	candidates: &[String],
	minimum_score: f32,
	limit: usize,
) -> Option<Vec<String>> {
	fuzzy_best_n_main(query, candidates, minimum_score, limit, true)
}

pub fn fuzzy_best_n_main(
	query: &str,
	candidates: &[String],
	minimum_score: f32,
	limit: usize,
	substring: bool,
) -> Option<Vec<String>> {
	let mut results: Vec<Match> = candidates
		.iter()
		.filter_map(|candidate| {
			let score = trigram_edit_fuzzy_score_main(query, candidate, substring);

			if score < 0.01 {
				None
			} else {
				Some(Match {
					text: candidate,
					score,
				})
			}
		})
		.collect();

	if results.is_empty() {
		return None;
	}

	results.sort_unstable_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(Ordering::Equal));

	let best = results.first().map(|m| m.score).unwrap();
	if best < minimum_score {
		return None;
	}
	// minimum tolerance allowed:
	let bottom_line = (best - 0.15).max(minimum_score);

	results.retain(|m| m.score >= bottom_line);
	results.truncate(limit);
	#[cfg(debug_assertions)]
	{
		eprintln!("Fuzzy search results for query '{query}':");
		for m in &results {
			eprintln!(" - '{}' (score: {:.2})", m.text, m.score);
		}
	}
	Some(results.into_iter().map(|m| m.text.to_string()).collect())
}

pub fn trigram_substring_edit_fuzzy_score(query: &str, text: &str) -> f32 {
	trigram_edit_fuzzy_score_main(query, text, true)
}

pub fn trigram_edit_fuzzy_score(query: &str, text: &str) -> f32 {
	trigram_edit_fuzzy_score_main(query, text, false)
}

/// A more sophisticated fuzzy score that combines trigram similarity, edit
/// distance, and bonuses for substring matches
fn trigram_edit_fuzzy_score_main(query: &str, text: &str, substring: bool) -> f32 {
	if query.is_empty() || text.is_empty() {
		return 0.0;
	}

	let query = query.to_lowercase();
	let text = text.to_lowercase();

	let mut too_short = false;
	let q_len = query.chars().count();
	let t_len = text.chars().count();

	// not suitable for trigram
	if q_len < 5 || t_len < 5 {
		too_short = true;
	}

	let tri = if too_short {
		0.0
	} else {
		trigram_jaccard_score(&query, &text)
	};
	// early rejection
	if !too_short && tri < 0.01 {
		#[cfg(debug_assertions)]
		{
			eprintln!("Early rejection comparing\n - '{query}'\n - '{text}'\n score: {tri}");
		}
		return tri;
	}

	// bonuses
	let mut bonus = 0.0;

	if text.contains(&query) {
		bonus += 0.05;
	}
	let words: Vec<&str> = text.split(|c: char| !c.is_alphanumeric()).collect();
	// word boundaries
	if words.iter().any(|w| w.ends_with(&query)) {
		bonus += 0.05;
	}
	if words.iter().any(|w| w.starts_with(&query)) {
		bonus += 0.05;
	}

	let edit = if substring && !too_short {
		best_substring_edit_score(&query, &text)
	} else {
		let dist = compare_string(&query, &text) as f32;
		(1.0 - dist / q_len.min(t_len) as f32).max(0.0)
	};

	let edit = if edit < (1.0 - get_dl_distance_percentage()) / 100.0 {
		0.5 * edit
	} else {
		edit
	};

	let score = if too_short {
		edit + bonus
	} else {
		(tri * 0.3 + edit * 0.7 + bonus).min(1.0)
	}
	.min(1.0);

	#[cfg(debug_assertions)]
	{
		eprintln!(
			"Comparing
- '{query}'
- '{text}'
 trigram score: {tri}
 edit score: {edit}
 bonus: {bonus}
 final score: {score}"
		);
	}

	score
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

fn fuzzy_match_cost(word: &str, dict_word: &str) -> Option<f32> {
	let dist = compare_string(word, dict_word);
	let max_len = word.len().max(dict_word.len());

	let norm = dist as f32 / max_len as f32;

	// perfect match
	if dist == 0 {
		return Some(0.0);
	}

	// partial match penalties
	if max_len <= 4 {
		if dist <= 1 && norm <= 0.35 {
			return Some(0.3);
		} else {
			return None;
		}
	}

	if max_len <= 7 {
		if dist <= 2 && norm <= 0.3 {
			return Some(0.4);
		} else {
			return None;
		}
	}

	if dist <= 3 && norm <= 0.25 {
		return Some(0.5);
	}

	None
}

#[allow(clippy::needless_range_loop)]
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
			if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
				matrix[i][j] = matrix[i][j].min(matrix[i - 2][j - 2] + 1);
			}
		}
	}

	matrix[a.len()][b.len()]
}

pub fn segment(input: &str, dict: &[String]) -> Vec<String> {
	let n = input.len();
	let mut best_at: Vec<Option<(f32, Vec<String>)>> = vec![None; n + 1];

	best_at[0] = Some((0.0, vec![]));

	for i in 1..=n {
		for j in 0..i {
			if let Some((prev_cost, prev_words)) = &best_at[j] {
				let mut word = &input[j..i];

				let mut best_cost = None;
				let mut best_idx = None;

				// fuzzy recovery: try to find a close match in the dictionary and use that instead of the original word
				for (idx, dict_word) in dict.iter().enumerate() {
					if let Some(cost) = fuzzy_match_cost(word, dict_word) {
						#[cfg(debug_assertions)]
						eprintln!(
							"[segment fuzzy] cost between '{word}' and '{dict_word}': {cost}"
						);

						best_cost = match best_cost {
							None => {
								best_idx = Some(idx);
								Some(cost)
							}
							Some(prev) => {
								// Some(prev.min(cost))
								if cost < prev {
									best_idx = Some(idx);
									Some(cost)
								} else {
									Some(prev)
								}
							}
						};
					}
				}

				if let Some(best_idx) = best_idx {
					word = &dict[best_idx];
				}

				let cost = if let Some(c) = best_cost {
					c
				} else if dict.contains(&word.to_string()) {
					0.0
				} else {
					// unknown word penalty
					1.0 + word.len() as f32 * 0.05
				};

				let total_cost = prev_cost + cost;

				let mut new_words = prev_words.clone();
				new_words.push(word.to_string());

				match &best_at[i] {
					None => best_at[i] = Some((total_cost, new_words)),
					Some((best_cost, _)) if total_cost < *best_cost => {
						best_at[i] = Some((total_cost, new_words))
					}
					_ => {}
				}
			}
		}
	}

	best_at[n]
		.clone()
		.map(|(_, words)| words)
		.unwrap_or_else(|| vec![input.to_string()])
}

/// segment but only for the first word, to recover the command itself, not the arguments
pub fn segment_1(input: &str, dict: &[String]) -> Vec<Vec<String>> {
	let mut candidates: Vec<(f32, Vec<String>)> = Vec::new();

	for split in 1..input.len() {
		let head = &input[..split];
		let tail = &input[split..];

		for dict_word in dict {
			if let Some(cost) = fuzzy_match_cost(head, dict_word) {
				let tail_penalty = tail.len() as f32 * 0.05;

				let total_cost = cost + tail_penalty;
				candidates.push((total_cost, vec![dict_word.clone(), tail.to_string()]));
			}
		}
	}

	// full match
	for dict_word in dict {
		if let Some(cost) = fuzzy_match_cost(input, dict_word) {
			candidates.push((cost, vec![dict_word.clone()]));
		}
	}

	// no match at all
	if candidates.is_empty() {
		return vec![vec![input.to_string()]];
	}

	candidates.sort_unstable_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal));

	let best_cost = candidates.first().map(|c| c.0).unwrap_or(0.0);
	// filter candidates that are significantly worse than the best
	let baseline = best_cost + 0.1; // allow some tolerance
	candidates.retain(|c| c.0 <= baseline);

	let limit = 9;

	candidates
		.into_iter()
		.take(limit)
		.map(|(_, words)| words)
		.collect()
}

mod tests {
	#[allow(unused_imports)]
	use super::*;

	#[test]
	fn test_segment() {
		let dict = vec!["git", "commit", "vim"]
			.into_iter()
			.map(String::from)
			.collect::<Vec<String>>();

		let input = "gitcommit";
		let result = segment(input, &dict);
		assert_eq!(result, vec!["git", "commit"]);

		// test with typos
		let input = "gitcomit";
		let result = segment(input, &dict);
		assert_eq!(result, vec!["git", "commit"]);

		// preserve unknown words
		let input = "vimhelloworld";
		let result = segment(input, &dict);
		assert_eq!(result, vec!["vim", "helloworld"]);
	}
}
