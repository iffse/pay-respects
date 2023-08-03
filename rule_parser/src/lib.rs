use std::path::Path;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

#[proc_macro]
pub fn parse_rules(input: TokenStream) -> TokenStream {
	let directory = input.to_string().trim_matches('"').to_owned();
	let rules = get_rules(directory);

	gen_match_rules(rules)
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

fn get_rules(directory: String) -> Vec<Rule> {
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

fn gen_match_rules(rules: Vec<Rule>) -> TokenStream {
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
						.map(|x| x.to_lowercase())
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
			};
			let match_tokens = quote! {
				#(#pattern_suggestions)*
			};

			suggestion_tokens.push(match_tokens);

			let string_patterns = pattern.join("\", \"");
			let string_patterns: TokenStream2 = format!("vec![\"{}\"]", string_patterns).parse().unwrap();
			patterns_tokens.push(string_patterns);
		}

		matches_tokens.push(quote!{
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
				return None;
				}
				)*
				_ => { return None; }
		};
	}.into()
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

			eval_conditions.push(quote!{#evaluated_condition == !#reverse});
		}
		let suggest = lines.join("\n");
		return (suggest, eval_conditions);
	}
	(suggest.to_owned(), vec![quote!{true}])
}

fn eval_condition(condition: &str, arg: &str) -> TokenStream2 {
	match condition {
		"executable" => {
			quote!{
				std::process::Command::new("which")
					.arg(#arg)
					.output()
					.expect("failed to execute process")
					.status
					.success()
			}
		},
		"err_contains" => quote!{error_msg.contains(#arg)},
		"cmd_contains" => quote!{command.contains(#arg)},
		_ => unreachable!("Unknown condition when evaluation condition: {}", condition),
	}
}

fn eval_suggest(suggest: &str) -> TokenStream2 {
	let mut suggest = suggest.to_owned();
	let rtag = |x: i32, y: String| {
		let tag = format!("tag{} = {}", x, y);
		let tag: TokenStream2 = tag.parse().unwrap();
		tag
	};

	let tag = |x: i32| {
		let tag = format!("{{tag{}}}", x);
		let tag = tag.as_str();
		let tag = tag.to_owned();
		tag
	};
	let mut replace_tag = 0;
	let mut replace_list = Vec::new();

	if suggest.contains("{{command}}") {
		let command = "last_command".to_string();

		replace_list.push(rtag(replace_tag, command));
		suggest = suggest.replace("{{command}}", &tag(replace_tag));
		replace_tag += 1;
	}

	let mut opt_lists = Vec::new();
	while suggest.contains("{{opt::") {
		let placeholder_start = "{{opt::";
		let placeholder_end = "}}";

		let start_index = suggest.find(placeholder_start).unwrap();
		let end_index = suggest[start_index..].find(placeholder_end).unwrap()
			+ start_index
			+ placeholder_end.len();

		let placeholder = start_index..end_index;

		let args = start_index + placeholder_start.len()..end_index - placeholder_end.len();
		let opt = &suggest[args.to_owned()];
		let regex = opt.trim();
		let current_tag = tag(replace_tag);
		let token_tag: TokenStream2 = format!("tag{}", replace_tag).parse().unwrap();
		let command = quote! {
			let #token_tag = opt_regex(#regex, &mut last_command);
		};
		opt_lists.push(command);

		replace_list.push(rtag(replace_tag, current_tag.to_owned()));
		suggest.replace_range(placeholder, &current_tag);
		replace_tag += 1;
	}

	while suggest.contains("{{command") {
		let placeholder_start = "{{command";
		let placeholder_end = "}}";

		let start_index = suggest.find(placeholder_start).unwrap();
		let end_index = suggest[start_index..].find(placeholder_end).unwrap()
			+ start_index
			+ placeholder_end.len();

		let placeholder = start_index..end_index;

		let args = start_index + placeholder_start.len()..end_index - placeholder_end.len();
		let range = suggest[args.to_owned()].trim_matches(|c| c == '[' || c == ']');
		if let Some((start, end)) = range.split_once(':') {
			let mut start_string = start.to_string();
			let start = start.parse::<i32>().unwrap_or(0);
			if start < 0 {
				start_string = format!("split_command.len() {}", start);
			};
			let end_string;
			let parsed_end = end.parse::<i32>();
			if parsed_end.is_err() {
				end_string = String::from("split_command.len()");
			} else {
				let end = parsed_end.clone().unwrap();
				if end < 0 {
					end_string = format!("split_command.len() {}", end + 1);
				} else {
					end_string = (end + 1).to_string();
				}
			};

			let command = format!{r#"split_command[{}..{}].join(" ")"#, start_string, end_string};

			replace_list.push(rtag(replace_tag, command));
			suggest.replace_range(placeholder, &tag(replace_tag));
			replace_tag += 1;
		} else {
			let range = range.parse::<i32>().unwrap_or(0);
			let command = format!("split_command[{}]", range);

			replace_list.push(rtag(replace_tag, command));
			suggest.replace_range(placeholder, &tag(replace_tag));
			replace_tag += 1;
		}
	}

	while suggest.contains("{{typo") {
		let placeholder_start = "{{typo";
		let placeholder_end = "}}";

		let start_index = suggest.find(placeholder_start).unwrap();
		let end_index = suggest[start_index..].find(placeholder_end).unwrap()
			+ start_index
			+ placeholder_end.len();

		let placeholder = start_index..end_index;
		let args = start_index + placeholder_start.len()..end_index - placeholder_end.len();

		let string_index;
		if suggest.contains('[') {
			let split = suggest[args.to_owned()]
				.split(&['[', ']'])
				.collect::<Vec<&str>>();
			let command_index = split[1].parse::<i32>().unwrap();
			if command_index < 0 {
				// command_index += split_command.len() as i32;
				string_index = format!("split_command.len() {}", command_index);
			} else {
				string_index = command_index.to_string();
			}
		} else {
			unreachable!("Typo suggestion must have a command index");
		}
		let mut match_list = Vec::new();
		if suggest.contains('(') {
			let split = suggest[args.to_owned()]
				.split(&['(', ')'])
				.collect::<Vec<&str>>();
			match_list = split[1].trim().split(',').collect::<Vec<&str>>();
		}

		let match_list = match_list
			.iter()
			.map(|s| s.trim().to_string())
			.collect::<Vec<String>>();
		let string_match_list = match_list.join(r#"".to_string(), ""#);
		let string_match_list = format!(r#""{}".to_string()"#, string_match_list);

		let command = format!("suggest_typo(&split_command[{}], vec![{}])", string_index, string_match_list);

		replace_list.push(rtag(replace_tag, command));
		suggest.replace_range(placeholder, &tag(replace_tag));
		replace_tag += 1;
	}

	quote! {
		#(#opt_lists)*
		let split_command = split_command(&last_command);
		return Some(format!{#suggest, #(#replace_list),*});
	}
}
