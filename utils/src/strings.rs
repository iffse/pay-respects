// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use colored::*;

const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");

pub fn print_warning(message: &str) {
	eprintln!("{}: {}", PROJECT_NAME.yellow(), message);
}

pub fn print_error(message: &str) {
	eprintln!("{}: {}", PROJECT_NAME.red().bold(), message);
}

pub fn unexpected_format(message: &str) {
	print_error(&format!("Unexpected format: {}", message));
}

pub fn log_string(debug_level: usize, message: &str) -> String {
	format!("[{}]: {}", debug_level.to_string().blue(), message)
}

pub fn log_plain(debug_level: usize, message: &str) -> String {
	format!("[{}]: {}", debug_level, message)
}

pub fn format_prefix(prefix: &str, string: &str) -> String {
	let indent_count = prefix.chars().count();
	let indent = format!("{} ", " ".repeat(indent_count));
	let stripped = string
		.lines()
		.map(|line| line.trim())
		.collect::<Vec<_>>()
		.join("\n");
	let string = stripped.replace("\n", &format!("\n{}", indent));
	format!("{} {}", prefix.cyan().bold(), string)
}

pub fn remove_color_codes(input: &str) -> String {
	let re = regex_lite::Regex::new(r"\x1b\[[0-9;]*m").unwrap();
	re.replace_all(input, "").to_string()
}

/// Replaces all occurrences of the target character in the input string with
/// the replacement string, but only if the target character is not escaped
/// by an **odd number** of backslashes.
pub fn replace_unescaped_character(input: &str, target: char, replacement: &str) -> String {
	let mut result = String::with_capacity(input.len());
	let mut backslash_count = 0;

	for c in input.chars() {
		match c {
			'\\' => {
				backslash_count += 1;
				result.push(c);
			}
			_ => {
				if c == target {
					if backslash_count % 2 == 0 {
						// non-escaped character
						result.push_str(replacement);
					} else {
						// escaped character, keep it as is
						result.push(c);
					}
				} else {
					backslash_count = 0;
					result.push(c);
				}
			}
		}
	}
	result
}

/// Same as `replace_unescaped_character`, but for escaped characters (i.e.,
/// those preceded by an odd number of backslashes).
pub fn replace_escaped_character(input: &str, target: char, replacement: &str) -> String {
	let mut result = String::with_capacity(input.len());
	let mut backslash_count = 0;

	for c in input.chars() {
		match c {
			'\\' => {
				backslash_count += 1;
				result.push(c);
			}
			_ => {
				if c == target {
					if backslash_count % 2 == 0 {
						// non-escaped character, keep it as is
						result.push(c);
					} else {
						// escaped character
						result.pop(); // remove the escaping backslash
						result.push_str(replacement);
					}
				} else {
					backslash_count = 0;
					result.push(c);
				}
			}
		}
	}
	result
}

pub fn split_unescaped_character(input: &str, char: char) -> Vec<String> {
	let replaced = replace_unescaped_character(input, char, "\0");
	replaced.split('\0').map(|s| s.to_string()).collect()
}
