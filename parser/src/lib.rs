// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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

#[derive(serde::Deserialize)]
struct Rule {
	command: String,
	match_err: Vec<MatchError>,
}

#[derive(serde::Deserialize)]
struct MatchError {
	pattern: Vec<String>,
	suggest: Vec<String>,
}

fn get_rules(directory: &str) -> Vec<Rule> {
	let files = std::fs::read_dir(directory).expect("Failed to read directory.");

	let mut rules = Vec::new();
	for file in files {
		let file = file.expect("Failed to read file.");
		let path = file.path();
		let path = path.to_str().expect("Failed to convert path to string.");

		let rule_file = parse_file(Path::new(path));
		rules.push(rule_file);
	}
	rules
}

fn gen_match_rules(rules: &[Rule]) -> TokenStream {
	let command = rules
		.iter()
		.map(|x| x.command.to_owned())
		.collect::<Vec<String>>();
	let command_matches = rules
		.iter()
		.map(|x| {
			x.match_err
				.iter()
				.map(|x| {
					let pattern = x
						.pattern
						.iter()
						.map(|x| x.to_lowercase())
						.collect::<Vec<String>>();
					let suggests = x
						.suggest
						.iter()
						.map(|x| x.to_string())
						.collect::<Vec<String>>();
					(pattern, suggests)
				})
				.collect::<Vec<(Vec<String>, Vec<String>)>>()
		})
		.collect::<Vec<Vec<(Vec<String>, Vec<String>)>>>();

	let mut matches_tokens = Vec::new();

	for match_err in command_matches {
		let mut suggestion_tokens = Vec::new();
		let mut patterns_tokens = Vec::new();
		for (pattern, suggests) in match_err {
			// let mut match_condition = Vec::new();
			let mut pattern_suggestions = Vec::new();
			for suggest in suggests {
				let (suggestion_no_condition, conditions) = parse_conditions(&suggest);
				let suggest = eval_suggest(&suggestion_no_condition);
				let suggestion = quote! {
					if #(#conditions)&&* {
						#suggest;
					};
				};
				pattern_suggestions.push(suggestion);
			}
			let match_tokens = quote! {
				#(#pattern_suggestions)*
			};

			suggestion_tokens.push(match_tokens);

			let string_patterns = pattern.join("\", \"");
			let string_patterns: TokenStream2 =
				format!("[\"{}\"]", string_patterns).parse().unwrap();
			patterns_tokens.push(string_patterns);
		}

		matches_tokens.push(quote! {
			#(
			for pattern in #patterns_tokens {
				if error_msg.contains(pattern) {
					#suggestion_tokens;
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

fn parse_file(file: &Path) -> Rule {
	let file = std::fs::read_to_string(file).expect("Failed to read file.");
	toml::from_str(&file).expect("Failed to parse toml.")
}

fn parse_conditions(suggest: &str) -> (String, Vec<TokenStream2>) {
	let mut eval_conditions = Vec::new();
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
			let evaluated_condition = eval_condition(condition, arg);

			eval_conditions.push(quote! {#evaluated_condition == !#reverse});
		}
		let suggest = lines.join("\n");
		return (suggest, eval_conditions);
	}
	(suggest.to_owned(), vec![quote! {true}])
}

fn eval_condition(condition: &str, arg: &str) -> TokenStream2 {
	match condition {
		"executable" => quote! {executables.contains(&#arg.to_string())},
		"err_contains" => quote! {error_msg.contains(#arg)},
		"cmd_contains" => quote! {last_command.contains(#arg)},
		"exe_contains" => quote! {split[0].contains(#arg)},
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
