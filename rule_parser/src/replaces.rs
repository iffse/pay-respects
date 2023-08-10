use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

fn rtag(name: &str, x: i32, y: String) -> TokenStream2 {
	let tag = format!("{}{} = {}", name, x, y);
	let tag: TokenStream2 = tag.parse().unwrap();
	tag
}

fn tag(name: &str, x: i32) -> String {
	let tag = format!("{{{}{}}}", name, x);
	let tag = tag.as_str();
	let tag = tag.to_owned();
	tag
}

fn eval_placeholder(
	string: &str,
	start: &str,
	end: &str,
) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
	let start_index = string.find(start).unwrap();
	let end_index = string[start_index..].find(end).unwrap() + start_index + end.len();

	let placeholder = start_index..end_index;

	let args = start_index + start.len()..end_index - end.len();

	(placeholder, args)
}

pub fn opts(
	suggest: &mut String,
	replace_list: &mut Vec<TokenStream2>,
	opt_list: &mut Vec<TokenStream2>,
) {
	let mut replace_tag = 0;
	let tag_name = "opts";
	while suggest.contains("{{opt::") {
		let (placeholder, args) = eval_placeholder(suggest, "{{opt::", "}}");

		let opt = &suggest[args.to_owned()];
		let regex = opt.trim();
		let current_tag = tag(tag_name, replace_tag);
		let token_tag: TokenStream2 = format!("{}{}", tag_name, replace_tag).parse().unwrap();
		let command = quote! {
			let #token_tag = opt_regex(#regex, &mut last_command);
		};
		opt_list.push(command);

		replace_list.push(rtag(tag_name, replace_tag, current_tag.to_owned()));
		suggest.replace_range(placeholder, &current_tag);
		replace_tag += 1;
	}
}

pub fn cmd_reg(suggest: &mut String, replace_list: &mut Vec<TokenStream2>) {
	let mut replace_tag = 0;
	let tag_name = "cmd";

	while suggest.contains("{{cmd::") {
		let (placeholder, args) = eval_placeholder(suggest, "{{cmd::", "}}");

		let regex = suggest[args.to_owned()].trim();

		let command = format!("cmd_regex(r###\"{}\"###, &last_command)", regex);

		replace_list.push(rtag(tag_name, replace_tag, command));
		suggest.replace_range(placeholder, &tag(tag_name, replace_tag));
		replace_tag += 1;
	}
}

pub fn err(suggest: &mut String, replace_list: &mut Vec<TokenStream2>) {
	let mut replace_tag = 0;
	let tag_name = "err";

	while suggest.contains("{{err::") {
		let (placeholder, args) = eval_placeholder(suggest, "{{err::", "}}");

		let regex = suggest[args.to_owned()].trim();

		let command = format!("err_regex(r###\"{}\"###, error_msg)", regex);

		replace_list.push(rtag(tag_name, replace_tag, command));
		suggest.replace_range(placeholder, &tag(tag_name, replace_tag));
		replace_tag += 1;
	}
}

pub fn command(suggest: &mut String, replace_list: &mut Vec<TokenStream2>) {
	let mut replace_tag = 0;
	let tag_name = "command";
	while suggest.contains("{{command") {
		let (placeholder, args) = eval_placeholder(suggest, "{{command", "}}");

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

			let command = format! {r#"split_command[{}..{}].join(" ")"#, start_string, end_string};

			replace_list.push(rtag(tag_name, replace_tag, command));
			suggest.replace_range(placeholder, &tag(tag_name, replace_tag));
		} else {
			let range = range.parse::<i32>().unwrap_or(0);
			let command = format!("split_command[{}]", range);

			replace_list.push(rtag(tag_name, replace_tag, command));
			suggest.replace_range(placeholder, &tag(tag_name, replace_tag));
		}
		replace_tag += 1;
	}
}

pub fn typo(suggest: &mut String, replace_list: &mut Vec<TokenStream2>) {
	let mut replace_tag = 0;
	let tag_name = "typo";

	while suggest.contains("{{typo") {
		let (placeholder, args) = eval_placeholder(suggest, "{{typo", "}}");

		let string_index = if suggest.contains('[') {
			let split = suggest[args.to_owned()]
				.split(&['[', ']'])
				.collect::<Vec<&str>>();
			let command_index = split[1];
			if !command_index.contains(':') {
				let command_index = command_index.parse::<i32>().unwrap();

				let index = if command_index < 0 {
					format!("split_command.len() {}", command_index)
				} else {
					command_index.to_string()
				};
				format!("{}..{} + 1", index, index)
			} else {
				let (start, end) = command_index.split_once(':').unwrap();
				let start = start.parse::<i32>().unwrap_or(0);
				let start_string = if start < 0 {
					format!("split_command.len() {}", start)
				} else {
					start.to_string()
				};
				let end = end.parse::<i32>();
				let end_string = if end.is_err() {
					String::from("split_command.len()")
				} else {
					let end = end.unwrap();
					if end < 0 {
						format!("split_command.len() {}", end + 1)
					} else {
						(end + 1).to_string()
					}
				};

				format!("{}..{}", start_string, end_string)
			}
		} else {
			unreachable!("Typo suggestion must have a command index");
		};
		let match_list;
		if suggest.contains('(') {
			let split = suggest[args.to_owned()]
				.split_once("(")
				.unwrap()
				.1
				.rsplit_once(")")
				.unwrap()
				.0;
			match_list = split.split(',').collect::<Vec<&str>>();
		} else {
			unreachable!("Typo suggestion must have a match list");
		}

		let match_list = match_list
			.iter()
			.map(|s| s.trim().to_string())
			.collect::<Vec<String>>();

		let command;
		if match_list[0].starts_with("eval_shell_command(") {
			let function = match_list.join(",");
			// add a " after first comma, and a " before last )
			let function = format!(
				"{}\"{}{}",
				&function[..function.find(',').unwrap() + 1],
				&function[function.find(',').unwrap() + 1..function.len() - 1],
				"\")"
			);
			command = format!(
				"suggest_typo(&split_command[{}], &{})",
				string_index, function
			);
		} else {
			let match_list = match_list
				.iter()
				.map(|s| s.trim().to_string())
				.collect::<Vec<String>>();
			let string_match_list = match_list.join("\".to_string(), \"");
			let string_match_list = format!("\"{}\".to_string()", string_match_list);
			command = format!(
				"suggest_typo(&split_command[{}], &[{}])",
				string_index, string_match_list
			);
		}

		replace_list.push(rtag(tag_name, replace_tag, command));
		suggest.replace_range(placeholder, &tag(tag_name, replace_tag));
		replace_tag += 1;
	}
}

pub fn shell(suggest: &mut String, cmd_list: &mut Vec<String>) {
	while suggest.contains("{{shell") {
		let (placeholder, args) = eval_placeholder(suggest, "{{shell", "}}");
		let range = suggest[args.to_owned()].trim_matches(|c| c == '(' || c == ')');

		let command = format!("eval_shell_command(shell, {})", range);

		suggest.replace_range(placeholder, &command);
		cmd_list.push(command);
	}
}

pub fn shell_tag(
	suggest: &mut String,
	replace_list: &mut Vec<TokenStream2>,
	cmd_list: Vec<String>,
) {
	let mut replace_tag = 0;
	let tag_name = "shell";

	for command in cmd_list {
		if suggest.contains(&command) {
			*suggest = suggest.replace(&command, &tag(tag_name, replace_tag));

			let split = command.split_once(',').unwrap();
			let argument = split.1.trim_end_matches(')').trim();
			let argument = format!("\"{}\"", argument);
			let function = format!("{}, {}).join(\"\")", split.0, argument);
			// let function = format!("\"{}, {}\"", split.0, split.1);
			replace_list.push(rtag(tag_name, replace_tag, function));
			replace_tag += 1;
		}
	}
}
