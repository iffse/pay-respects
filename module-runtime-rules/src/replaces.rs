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

		let opts = {
			let caps = opt_regex(regex, last_command);
			if caps.is_empty() {
				"".to_string()
			} else {
				format!(" {}", caps)
			}
		};

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
			if let Ok(end) = parsed_end {
				end_index = end;
				if end_index < 0 {
					end_index += split_command.len() as i32 + 1;
				} else {
					end_index += 1;
				}
			} else {
				end_index = split_command.len() as i32;
			}

			let command = split_command[start_index as usize..end_index as usize].join(" ");

			suggest.replace_range(placeholder, &command);
		} else {
			let range = range.parse::<usize>().unwrap_or(0);
			let command = &split_command[range];

			suggest.replace_range(placeholder, command);
		}
	}
}

pub fn typo(suggest: &mut String, split_command: &[String], executables: &[String]) {
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
				let end_index = if let Ok(end) = end {
					if end < 0 {
						split_command.len() as i32 + end + 1
					} else {
						end + 1
					}
				} else {
					split_command.len() as i32
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
			split.split(&[',', '\n']).collect::<Vec<&str>>()
		} else {
			unreachable!("Typo suggestion must have a match list");
		};

		let match_list = match_list
			.iter()
			.map(|s| s.trim().to_string())
			.collect::<Vec<String>>();

		let command = suggest_typo(&split_command[index], &match_list, executables);

		suggest.replace_range(placeholder, &command);
	}
}

pub fn select(suggest: &mut String, split_command: &[String], select_list: &mut Vec<String>) {
	if suggest.contains("{{select") {
		let (placeholder, args) = eval_placeholder(suggest, "{{select", "}}");

		let _ = if suggest.contains('[') {
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
				unreachable!("Select suggestion does not support range");
			}
		} else {
			0
		};

		let selection_list = if suggest.contains('(') {
			let split = suggest[args.to_owned()]
				.split_once("(")
				.unwrap()
				.1
				.rsplit_once(")")
				.unwrap()
				.0;
			split.split(&[',', '\n']).collect::<Vec<&str>>()
		} else {
			unreachable!("Select suggestion must have a match list");
		};

		let selection_list = selection_list
			.iter()
			.map(|s| s.trim().to_string())
			.collect::<Vec<String>>();

		let selects = selection_list;

		for match_ in selects {
			select_list.push(match_);
		}

		let tag = "{{selection}}";
		let placeholder = suggest[placeholder.clone()].to_owned();
		*suggest = suggest.replace(&placeholder, tag);
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
