// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use itertools::sorted_unstable;
use pay_respects_utils::strings::split_unescaped_character;
use std::path::Path;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod replaces;

#[proc_macro]
pub fn parse_rules(input: TokenStream) -> TokenStream {
	let rules = get_rules(input.to_string().trim_matches('"'));
	gen_match_rules(&rules)
}

#[proc_macro]
pub fn parse_inline_rules(input: TokenStream) -> TokenStream {
	let rules = get_rules(input.to_string().trim_matches('"'));
	gen_inline_rules(&rules)
}

#[derive(serde::Deserialize)]
struct Rule {
	command: String,
	extends: Option<Vec<String>>,
	match_err: Vec<MatchError>,
}

#[derive(serde::Deserialize)]
struct MatchError {
	pattern: Option<Vec<String>>,
	suggest: Vec<String>,
}

fn get_rules(directory: &str) -> Vec<Rule> {
	let files = std::fs::read_dir(directory)
		.expect("Failed to read directory.")
		.map(|entry| {
			let entry = entry.expect("Failed to read directory entry.");
			entry
				.path()
				.to_str()
				.expect("Failed to convert path to string.")
				.to_string()
		});
	let files = sorted_unstable(files).collect::<Vec<String>>();

	let mut rules = Vec::new();
	for path in files {
		let rule_file = parse_file(Path::new(&path));
		rules.push(rule_file);
	}
	rules
}

fn gen_match_rules(rules: &[Rule]) -> TokenStream {
	let command = rule_commands(rules);
	let command_matches = parse_match_err(rules);

	let mut matches_tokens = Vec::new();

	for match_err in command_matches {
		let mut suggestion_tokens = Vec::new();
		let mut patterns_tokens = Vec::new();
		for (pattern, suggests) in match_err {
			// let mut match_condition = Vec::new();
			let mut pattern_suggestions = Vec::new();
			for suggest in suggests {
				let (suggestion_no_condition, mut conditions) = parse_conditions(&suggest);
				if let Some(conditions) = &mut conditions {
					conditions.retain(|x| x != "INLINE");
				}

				let suggestion = parse_suggestion(&suggestion_no_condition, conditions);
				pattern_suggestions.push(suggestion);
			}
			let match_tokens = quote! {
				#(#pattern_suggestions)*
			};

			suggestion_tokens.push(match_tokens);

			if let Some(pattern) = pattern {
				let string_patterns = pattern.join("\"###, r###\"");
				let string_patterns: TokenStream2 =
					format!("[r###\"{}\"###]", string_patterns).parse().unwrap();
				patterns_tokens.push(string_patterns);
			} else {
				patterns_tokens.push("[\"\"]".parse().unwrap());
			}
		}

		matches_tokens.push(quote! {
			#(
			for pattern in #patterns_tokens {
				if error_lower.contains(pattern) {
					#suggestion_tokens;
					break;
				};
			})*
		})
	}
	quote! {
		let mut last_command = last_command.to_string();
		match executable {
			#(
			#command => {
				#matches_tokens
				}
				)*
				_ => {}
		};
	}
	.into()
}

fn gen_inline_rules(rules: &[Rule]) -> TokenStream {
	let command = rule_commands(rules);
	let command_matches = parse_match_err(rules);

	let mut matches_tokens = Vec::new();

	for match_err in command_matches {
		let mut suggestion_tokens = Vec::new();
		for (_, suggests) in match_err {
			// let mut match_condition = Vec::new();
			let mut pattern_suggestions = Vec::new();
			for suggest in suggests {
				let (suggestion_no_condition, mut conditions) = parse_conditions(&suggest);
				if conditions.is_none() {
					continue;
				}
				if let Some(conditions) = &mut conditions {
					if !conditions.contains(&"INLINE".to_string()) {
						continue;
					}
					conditions.retain(|x| {
						x != "INLINE" && !x.starts_with("err") && !x.starts_with("!err")
					});
				}

				let suggestion = parse_suggestion(&suggestion_no_condition, conditions);
				pattern_suggestions.push(suggestion);
			}
			let match_tokens = quote! {
				#(#pattern_suggestions)*
			};

			if match_tokens.is_empty() {
				continue;
			}
			suggestion_tokens.push(match_tokens);
		}

		matches_tokens.push(quote! {
			#(
				#suggestion_tokens;
			)*
		})
	}
	quote! {
		let mut last_command = last_command.to_string();
		match executable {
			#(
			#command => {
				#matches_tokens
				}
				)*
				_ => {}
		};
	}
	.into()
}

#[allow(clippy::type_complexity)]
fn parse_match_err(rules: &[Rule]) -> Vec<Vec<(Option<Vec<String>>, Vec<String>)>> {
	rules
		.iter()
		.map(|x| {
			x.match_err
				.iter()
				.map(|x| {
					let pattern = if let Some(pattern) = &x.pattern {
						let pattern = pattern
							.iter()
							.map(|x| x.to_lowercase())
							.collect::<Vec<String>>();
						Some(pattern)
					} else {
						None
					};
					let suggests = x
						.suggest
						.iter()
						.map(|x| x.to_string())
						.collect::<Vec<String>>();
					(pattern, suggests)
				})
				.collect::<Vec<(Option<Vec<String>>, Vec<String>)>>()
		})
		.collect::<Vec<Vec<(Option<Vec<String>>, Vec<String>)>>>()
}

fn rule_commands(rules: &[Rule]) -> Vec<TokenStream2> {
	rules
		.iter()
		.map(|x| {
			if let Some(extends) = &x.extends {
				format!(
					"\"{}\"|{}",
					x.command,
					extends
						.iter()
						.map(|x| format!("\"{}\"", x))
						.collect::<Vec<String>>()
						.join("|")
				)
				.parse()
				.unwrap()
			} else {
				format!("\"{}\"", x.command).parse().unwrap()
			}
		})
		.collect::<Vec<TokenStream2>>()
}

fn parse_file(file: &Path) -> Rule {
	let file = std::fs::read_to_string(file).expect("Failed to read file.");
	toml::from_str(&file).expect("Failed to parse toml.")
}

fn parse_conditions(suggest: &str) -> (String, Option<Vec<String>>) {
	if suggest.starts_with('#') {
		let mut lines = suggest.lines().collect::<Vec<&str>>();
		let mut conditions = String::new();
		for (i, line) in lines[0..].iter().enumerate() {
			conditions.push_str(line);
			if line.ends_with(']') {
				lines = lines[i + 1..].to_vec();
				break;
			}
		}

		let conditions = conditions
			.trim_start_matches(['#', '['])
			.trim_end_matches(']');
		let conditions = split_unescaped_character(conditions, ',')
			.into_iter()
			.map(|x| x.trim().to_string())
			.collect::<Vec<String>>();
		let suggest = lines.join("\n");
		return (suggest, Some(conditions));
	}
	(suggest.to_owned(), None)
}

fn tokenize_conditions(conditions: &[String]) -> Vec<TokenStream2> {
	let mut eval_conditions = Vec::new();
	for condition in conditions {
		let (mut condition, arg) = condition.split_once('(').unwrap();
		condition = condition.trim();
		// remove only the last character which is ')'
		// other ')' are kept for regex
		let arg = arg
			.to_string()
			.chars()
			.take(arg.len() - 1)
			.collect::<String>();

		let reverse = match condition.starts_with('!') {
			true => {
				condition = condition.trim_start_matches('!');
				true
			}
			false => false,
		};
		let evaluated_condition = eval_condition(condition, &arg);

		eval_conditions.push(quote! {#evaluated_condition == !#reverse});
	}
	eval_conditions
}

fn parse_suggestion(suggestion: &str, conditions: Option<Vec<String>>) -> TokenStream2 {
	if conditions.is_none() {
		return eval_suggest(suggestion);
	}
	let (conditions, is_function) = {
		let mut conditions = conditions.unwrap();
		if conditions.contains(&"FUNCTION".to_string()) {
			conditions.retain(|x| x != "FUNCTION");
			(conditions, true)
		} else {
			(conditions, false)
		}
	};

	let suggest = if is_function {
		let suggestion: TokenStream2 = suggestion.trim_matches('"').parse().unwrap();
		quote! {
			rules_function(#suggestion, &error_msg, &error_lower, &shell, &last_command, &executables, &split, &mut candidates, data);
		}
	} else {
		eval_suggest(suggestion)
	};

	if conditions.is_empty() {
		return suggest;
	}

	let conditions = tokenize_conditions(&conditions);
	quote! {
		if #(#conditions)&&* {
			#suggest
		}
	}
}

fn eval_condition(condition: &str, arg: &str) -> TokenStream2 {
	match condition {
		"executable" => quote! {executables.contains(&#arg.to_string())},
		"err_contains" => quote! {regex_match(#arg, &error_lower)},
		"cmd_contains" => quote! {regex_match(#arg, &last_command)},
		"min_length" => quote! {(split.len() >= #arg.parse::<usize>().unwrap())},
		"length" => quote! {(split.len() == #arg.parse::<usize>().unwrap())},
		"max_length" => quote! {(split.len() <= #arg.parse::<usize>().unwrap() + 1)},
		"shell" => quote! {(shell == #arg)},
		_ => unreachable!("Unknown condition when evaluation condition: {}", condition),
	}
}

fn eval_suggest(suggest: &str) -> TokenStream2 {
	let mut suggest = suggest.to_owned();
	if suggest.contains("{{command}}") {
		suggest = suggest.replace("{{command}}", "{last_command}");
	}

	let mut replace_list = Vec::new();
	let mut select_list = Vec::new();
	let mut opt_list = Vec::new();
	let mut cmd_list = Vec::new();

	replaces::opts(&mut suggest, &mut replace_list, &mut opt_list);
	replaces::cmd_reg(&mut suggest, &mut replace_list);
	replaces::err(&mut suggest, &mut replace_list);
	replaces::command(&mut suggest, &mut replace_list);
	replaces::shell(&mut suggest, &mut cmd_list);
	replaces::typo(&mut suggest, &mut replace_list);
	replaces::select(&mut suggest, &mut select_list);
	replaces::shell_tag(&mut suggest, &mut replace_list, &cmd_list);

	let suggests = if select_list.is_empty() {
		quote! {
			candidates.push(format!{#suggest, #(#replace_list),*});
		}
	} else {
		quote! {
			#(#select_list)*
			let suggest = format!{#suggest, #(#replace_list),*};
			for select in selects {
				let suggest = suggest.replace("{{selection}}", &select);
				candidates.push(suggest);
			}
		}
	};

	quote! {
		#(#opt_list)*
		#suggests
	}
}
