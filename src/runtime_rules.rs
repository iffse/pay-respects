use crate::replaces;
use crate::suggestions::*;

#[derive(serde::Deserialize)]
struct Rule {
	match_err: Vec<MatchError>,
}

#[derive(serde::Deserialize)]
struct MatchError {
	pattern: Vec<String>,
	suggest: Vec<String>,
}

pub fn runtime_match(
	executable: &str,
	last_command: &str,
	error_msg: &str,
	shell: &str,
) -> Option<String> {
	let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
		.unwrap_or_else(|_| std::env::var("HOME").unwrap() + "/.config");
	let rule_dir = format!("{}/pay-respects/rules", xdg_config_home);

	let file = format!("{}/{}.toml", rule_dir, executable);
	if !std::path::Path::new(&file).exists() {
		return None;
	}

	let file = std::fs::read_to_string(file).unwrap();
	let rule: Rule = toml::from_str(&file).unwrap();
	let split_command = split_command(&last_command);

	for match_err in rule.match_err {
		for pattern in match_err.pattern {
			if error_msg.contains(&pattern) {
				'suggest: for suggest in &match_err.suggest {
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
							if eval_condition(
								condition,
								arg,
								shell,
								last_command,
								error_msg,
								&split_command,
							) == reverse
							{
								continue 'suggest;
							}
						}

						// replacing placeholders
						let mut suggest = lines.join("\n").to_owned();
						if suggest.contains("{{command}}") {
							suggest = suggest.replace("{{command}}", last_command);
						}
						eval_suggest(&suggest, last_command, error_msg, shell, &split_command);
						return Some(suggest);
					}
				}
			}
		}
	}

	None
}

fn eval_condition(
	condition: &str,
	arg: &str,
	shell: &str,
	last_command: &str,
	error_msg: &str,
	split_command: &Vec<String>,
) -> bool {
	match condition {
		"executable" => check_executable(shell, arg),
		"err_contains" => error_msg.contains(arg),
		"cmd_contains" => last_command.contains(arg),
		"min_length" => split_command.len() >= arg.parse::<usize>().unwrap(),
		"length" => split_command.len() == arg.parse::<usize>().unwrap(),
		"max_length" => split_command.len() <= arg.parse::<usize>().unwrap() + 1,
		_ => unreachable!("Unknown condition when evaluation condition: {}", condition),
	}
}

fn eval_suggest(
	suggest: &str,
	last_command: &str,
	error_msg: &str,
	shell: &str,
	split_command: &Vec<String>,
) -> Option<String> {
	let mut suggest = suggest.to_owned();
	if suggest.contains("{{command}}") {
		suggest = suggest.replace("{{command}}", "{last_command}");
	}

	let mut last_command = last_command.to_owned();

	replaces::opts(&mut suggest, &mut last_command);
	replaces::cmd_reg(&mut suggest, &mut last_command);
	replaces::err(&mut suggest, error_msg);
	replaces::command(&mut suggest, split_command);
	replaces::shell(&mut suggest, shell);
	replaces::typo(&mut suggest, split_command);

	Some(suggest)
}
