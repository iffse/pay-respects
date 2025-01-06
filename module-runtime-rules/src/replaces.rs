use pay_respects_utils::evals::*;

fn tag(name: &str, x: i32) -> String {
	format!("{{{}{}}}", name, x)
}

pub fn eval_placeholder(
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

pub fn opts(suggest: &mut String, last_command: &mut String, opt_list: &mut Vec<(String, String)>) {
	let mut replace_tag = 0;
	let tag_name = "opts";

	while suggest.contains(" {{opt::") {
		let (placeholder, args) = eval_placeholder(suggest, " {{opt::", "}}");

		let opt = &suggest[args.to_owned()];
		let regex = opt.trim();
		let current_tag = tag(tag_name, replace_tag);

		let opts = format!(" {}", opt_regex(regex, last_command));

		opt_list.push((current_tag.clone(), opts));
		suggest.replace_range(placeholder, &current_tag);

		replace_tag += 1;
	}
}

pub fn cmd_reg(suggest: &mut String, last_command: &str) {
	while suggest.contains("{{cmd::") {
		let (placeholder, args) = eval_placeholder(suggest, "{{cmd::", "}}");

		let regex = suggest[args.to_owned()].trim();

		let command = cmd_regex(regex, last_command);
		suggest.replace_range(placeholder, &command)
	}
}

pub fn err(suggest: &mut String, error_msg: &str) {
	while suggest.contains("{{err::") {
		let (placeholder, args) = eval_placeholder(suggest, "{{err::", "}}");

		let regex = suggest[args.to_owned()].trim();

		let command = err_regex(regex, error_msg);
		suggest.replace_range(placeholder, &command)
	}
}

pub fn command(suggest: &mut String, split_command: &[String]) {
	while suggest.contains("{{command") {
		let (placeholder, args) = eval_placeholder(suggest, "{{command", "}}");

		let range = suggest[args.to_owned()].trim_matches(|c| c == '[' || c == ']');
		if let Some((start, end)) = range.split_once(':') {
			let mut start_index = start.parse::<i32>().unwrap_or(0);
			if start_index < 0 {
				start_index += split_command.len() as i32;
			};
			let mut end_index;
			let parsed_end = end.parse::<i32>();
			if parsed_end.is_err() {
				end_index = split_command.len() as i32;
			} else {
				end_index = parsed_end.unwrap();
				if end_index < 0 {
					end_index += split_command.len() as i32 + 1;
				} else {
					end_index += 1;
				}
			};

			let command = split_command[start_index as usize..end_index as usize].join(" ");

			suggest.replace_range(placeholder, &command);
		} else {
			let range = range.parse::<usize>().unwrap_or(0);
			let command = &split_command[range];

			suggest.replace_range(placeholder, command);
		}
	}
}

pub fn typo(suggest: &mut String, split_command: &[String], executables: &[String], shell: &str) {
	while suggest.contains("{{typo") {
		let (placeholder, args) = eval_placeholder(suggest, "{{typo", "}}");

		let index = if suggest.contains('[') {
			let split = suggest[args.to_owned()]
				.split(&['[', ']'])
				.collect::<Vec<&str>>();
			let command_index = split[1];
			if !command_index.contains(':') {
				let command_index = command_index.parse::<i32>().unwrap();

				let index = if command_index < 0 {
					split_command.len() as i32 + command_index
				} else {
					command_index
				};
				index as usize..index as usize + 1
			} else {
				let (start, end) = command_index.split_once(':').unwrap();
				let start = start.parse::<i32>().unwrap_or(0);
				let start_index = if start < 0 {
					split_command.len() as i32 + start
				} else {
					start
				};
				let end = end.parse::<i32>();
				let end_index = if end.is_err() {
					split_command.len() as i32
				} else {
					let end = end.unwrap();
					if end < 0 {
						split_command.len() as i32 + end + 1
					} else {
						end + 1
					}
				};

				start_index as usize..end_index as usize
			}
		} else {
			unreachable!("Typo suggestion must have a command index");
		};

		let match_list = if suggest.contains('(') {
			let split = suggest[args.to_owned()]
				.split_once("(")
				.unwrap()
				.1
				.rsplit_once(")")
				.unwrap()
				.0;
			split.split(',').collect::<Vec<&str>>()
		} else {
			unreachable!("Typo suggestion must have a match list");
		};

		let match_list = match_list
			.iter()
			.map(|s| s.trim().to_string())
			.collect::<Vec<String>>();

		let command = if match_list[0].starts_with("{{shell") {
			let function = match_list.join(",");
			let (_, args) = eval_placeholder(&function, "{{shell", "}}");
			let function = &function[args.to_owned()].trim_matches(|c| c == '(' || c == ')');
			suggest_typo(
				&split_command[index],
				&eval_shell_command(shell, function),
				executables,
			)
		} else {
			suggest_typo(&split_command[index], &match_list, executables)
		};

		suggest.replace_range(placeholder, &command);
	}
}

pub fn exes(suggest: &mut String, split_command: &[String], executables: &[String], exes_list: &mut Vec<String>) {
	if suggest.contains("{{exes") {
		let (placeholder, args) = eval_placeholder(suggest, "{{exes", "}}");

		let index = if suggest.contains('[') {
			let split = suggest[args.to_owned()]
				.split(&['[', ']'])
				.collect::<Vec<&str>>();
			let command_index = split[1];
			if !command_index.contains(':') {
				let command_index = command_index.parse::<i32>().unwrap();

				let index = if command_index < 0 {
					split_command.len() as i32 + command_index
				} else {
					command_index
				};
				index as usize
			} else {
				unreachable!("Exes suggestion does not support range");
			}
		} else {
			unreachable!("Exes suggestion must have a command index");
		};

		let matches = {
			let res = best_matches_path(&split_command[index], executables);
			if res.is_none() {
				vec![split_command[index].clone()]
			} else {
				res.unwrap()
			}
		};
		for match_ in matches {
			exes_list.push(match_);
		}

		let tag = "{{exes}}";
		let placeholder = suggest[placeholder.clone()].to_owned();
		*suggest = suggest.replace(&placeholder, &tag);
	}
}

pub fn shell(suggest: &mut String, shell: &str) {
	while suggest.contains("{{shell") {
		let (placeholder, args) = eval_placeholder(suggest, "{{shell", "}}");
		let range = suggest[args.to_owned()].trim_matches(|c| c == '(' || c == ')');

		let command = eval_shell_command(shell, range);

		suggest.replace_range(placeholder, &command.join("\n"));
	}
}
