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
	let file = get_rule(executable);
	file.as_ref()?;

	let file = std::fs::read_to_string(file.unwrap()).unwrap();
	let rule: Rule = toml::from_str(&file).unwrap();
	let split_command = split_command(last_command);

	let mut pure_suggest;

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

						pure_suggest = lines.join("\n").to_owned();
					} else {
						pure_suggest = suggest.to_owned();
					}
					// replacing placeholders
					if pure_suggest.contains("{{command}}") {
						pure_suggest = pure_suggest.replace("{{command}}", last_command);
					}
					return eval_suggest(&pure_suggest, last_command, error_msg, shell);
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
	split_command: &[String],
) -> bool {
	match condition {
		"executable" => check_executable(shell, arg),
		"err_contains" => error_msg.contains(arg),
		"cmd_contains" => last_command.contains(arg),
		"min_length" => split_command.len() >= arg.parse::<usize>().unwrap(),
		"length" => split_command.len() == arg.parse::<usize>().unwrap(),
		"max_length" => split_command.len() <= arg.parse::<usize>().unwrap() + 1,
		"shell" => shell == arg,
		_ => unreachable!("Unknown condition when evaluation condition: {}", condition),
	}
}

fn eval_suggest(suggest: &str, last_command: &str, error_msg: &str, shell: &str) -> Option<String> {
	let mut suggest = suggest.to_owned();
	if suggest.contains("{{command}}") {
		suggest = suggest.replace("{{command}}", "{last_command}");
	}

	let mut last_command = last_command.to_owned();
	let mut opt_list = Vec::new();

	replaces::opts(&mut suggest, &mut last_command, &mut opt_list);
	let split_command = split_command(&last_command);

	replaces::cmd_reg(&mut suggest, &last_command);
	replaces::err(&mut suggest, error_msg);
	replaces::command(&mut suggest, &split_command);
	replaces::shell(&mut suggest, shell);
	replaces::typo(&mut suggest, &split_command, shell);

	for (tag, value) in opt_list {
		suggest = suggest.replace(&tag, &value);
	}

	Some(suggest)
}

fn get_rule(executable: &str) -> Option<String> {
	let xdg_config_home = if cfg!(windows) {
		std::env::var("APPDATA").unwrap()
	} else {
		std::env::var("XDG_CONFIG_HOME")
			.unwrap_or_else(|_| std::env::var("HOME").unwrap() + "/.config")
	};

	let user_rule_dir = format!("{}/pay-respects/rules", xdg_config_home);
	let user_rule_file = format!("{}/{}.toml", user_rule_dir, executable);

	if std::path::Path::new(&user_rule_file).exists() {
		return Some(user_rule_file);
	}

	let check_dirs = |dirs: Vec<&str>| -> Option<String> {
		for dir in dirs {
			let rule_dir = format!("{}/pay-respects/rules", dir);
			let rule_file = format!("{}/{}.toml", rule_dir, executable);
			if std::path::Path::new(&rule_file).exists() {
				return Some(rule_file);
			}
		}
		None
	};

	let xdg_config_dirs = std::env::var("XDG_CONFIG_DIRS").unwrap_or("/etc/xdg".to_owned());
	let xdg_config_dirs = xdg_config_dirs.split(':').collect::<Vec<&str>>();

	if let Some(file) = check_dirs(xdg_config_dirs) {
		return Some(file);
	}

	let xdg_data_dirs =
		std::env::var("XDG_DATA_DIRS").unwrap_or("/usr/local/share:/usr/share".to_owned());
	let xdg_data_dirs = xdg_data_dirs.split(':').collect::<Vec<&str>>();

	if let Some(file) = check_dirs(xdg_data_dirs) {
		return Some(file);
	}

	None
}
