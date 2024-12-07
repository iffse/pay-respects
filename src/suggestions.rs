use std::io::stderr;
use std::process::{exit, Stdio};
use std::time::{Duration, Instant};

use colored::Colorize;
use inquire::*;
use regex_lite::Regex;

use crate::files::{get_best_match_file, get_path_files};
use crate::rules::match_pattern;
use crate::shell::{shell_evaluated_commands, Data};
use crate::style::highlight_difference;

pub fn suggest_candidates(data: &mut Data) {
	let executable = &data.split[0].to_string();
	let privilege = &data.privilege.clone();

	if privilege.is_none() {
		match_pattern("_PR_privilege", data);
	}
	match_pattern(executable, data);
	match_pattern("_PR_general", data);

	#[cfg(feature = "runtime-rules")]
	{
		use crate::runtime_rules::runtime_match;
		runtime_match(executable, data);
	}

	#[cfg(feature = "request-ai")]
	{
		if !data.candidates.is_empty() {
			return;
		}
		use crate::requests::ai_suggestion;
		use textwrap::{fill, termwidth};
		let command = &data.command;
		let split_command = &data.split;
		let error = &data.error.clone();

		// skip for commands with no arguments,
		// very likely to be an error showing the usage
		if privilege.is_some() && split_command.len() > 2
			|| privilege.is_none() && split_command.len() > 1
		{
			let suggest = ai_suggestion(command, error);
			if let Some(suggest) = suggest {
				let warn = format!("{}:", t!("ai-suggestion")).bold().blue();
				let note = fill(&suggest.note, termwidth());

				eprintln!("{}\n{}\n", warn, note);
				let command = suggest.command;
				data.add_candidate(&command);
			}
		}
	}
}

pub fn select_candidate(data: &mut Data) {
	let candidates = &data.candidates;
	if candidates.len() == 1 {
		let suggestion = candidates[0].to_string();
		let highlighted = highlight_difference(&data.shell, &suggestion, &data.command).unwrap();
		eprintln!("{}\n", highlighted);
		let confirm = format!("[{}]", t!("confirm-yes")).green();
		eprintln!("{}: {} {}", t!("confirm"), confirm, "[Ctrl+C]".red());
		std::io::stdin().read_line(&mut String::new()).unwrap();
		data.update_suggest(&suggestion);
		data.expand_suggest();
	} else {
		let mut highlight_candidates = candidates
			.iter()
			.map(|candidate| highlight_difference(&data.shell, candidate, &data.command).unwrap())
			.collect::<Vec<String>>();

		for candidate in highlight_candidates.iter_mut() {
			let lines = candidate.lines().collect::<Vec<&str>>();
			let mut formated = String::new();
			for (j, line) in lines.iter().enumerate() {
				if j == 0 {
					formated = line.to_string();
				} else {
					formated = format!("{}\n {}", formated, line);
				}
			}
			*candidate = formated;
		}

		let style = ui::Styled::default();
		let render_config = ui::RenderConfig::default()
			.with_prompt_prefix(style)
			.with_answered_prompt_prefix(style)
			.with_highlighted_option_prefix(style);

		let msg = format!("{}", t!("multi-suggest", num = candidates.len()))
			.bold()
			.blue();
		let confirm = format!("[{}]", t!("confirm-yes")).green();
		let hint = format!(
			"{} {} {}",
			"[↑/↓]".blue(),
			confirm,
			"[Ctrl+C]".red()
		);
		eprintln!("{}", msg);
		eprintln!("{}", hint);

		let ans = Select::new("\n", highlight_candidates.clone())
			.with_page_size(1)
			.without_filtering()
			.without_help_message()
			.with_render_config(render_config)
			.prompt()
			.unwrap();
		let pos = highlight_candidates.iter().position(|x| x == &ans).unwrap();
		let suggestion = candidates[pos].to_string();
		data.update_suggest(&suggestion);
		data.expand_suggest();
	}

	data.candidates.clear();
}

pub fn check_executable(shell: &str, executable: &str) -> bool {
	match shell {
		"nu" => std::process::Command::new(shell)
			.arg("-c")
			.arg(format!("if (which {} | is-empty) {{ exit 1 }}", executable))
			.output()
			.expect("failed to execute process")
			.status
			.success(),
		_ => std::process::Command::new(shell)
			.arg("-c")
			.arg(format!("command -v {}", executable))
			.output()
			.expect("failed to execute process")
			.status
			.success(),
	}
}

pub fn opt_regex(regex: &str, command: &mut String) -> String {
	let regex = Regex::new(regex).unwrap();

	let mut opts = Vec::new();
	for captures in regex.captures_iter(command) {
		for cap in captures.iter().skip(1).flatten() {
			opts.push(cap.as_str().to_owned());
		}
	}

	for opt in opts.clone() {
		*command = command.replace(&opt, "");
	}
	opts.join(" ")
}

pub fn err_regex(regex: &str, error_msg: &str) -> String {
	let regex = Regex::new(regex).unwrap();

	let mut err = Vec::new();
	for captures in regex.captures_iter(error_msg) {
		for cap in captures.iter().skip(1).flatten() {
			err.push(cap.as_str().to_owned());
		}
	}
	err.join(" ")
}

pub fn cmd_regex(regex: &str, command: &str) -> String {
	let regex = Regex::new(regex).unwrap();

	let mut cmd = Vec::new();
	for captures in regex.captures_iter(command) {
		for cap in captures.iter().skip(1).flatten() {
			cmd.push(cap.as_str().to_owned());
		}
	}
	cmd.join(" ")
}

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
		.collect::<Vec<String>>()
}

pub fn split_command(command: &str) -> Vec<String> {
	if cfg!(debug_assertions) {
		eprintln!("command: {command}")
	}
	// this regex splits the command separated by spaces, except when the space
	// is escaped by a backslash or surrounded by quotes
	let regex = r#"([^\s"'\\]+|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\\ )+|\\|\n"#;
	let regex = Regex::new(regex).unwrap();
	let split_command = regex
		.find_iter(command)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();
	split_command
}

pub fn suggest_typo(typos: &[String], candidates: Vec<String>) -> String {
	let mut path_files = Vec::new();
	let mut suggestions = Vec::new();
	for typo in typos {
		let typo = typo.as_str();
		if candidates.len() == 1 {
			match candidates[0].as_str() {
				"path" => {
					if path_files.is_empty() {
						path_files = get_path_files();
					};
					if let Some(suggest) = find_similar(typo, &path_files, Some(2)) {
						suggestions.push(suggest);
					} else {
						suggestions.push(typo.to_string());
					}
				}
				"file" => {
					if let Some(suggest) = get_best_match_file(typo) {
						suggestions.push(suggest);
					} else {
						suggestions.push(typo.to_string());
					}
				}
				_ => {}
			}
		} else if let Some(suggest) = find_similar(typo, &candidates, Some(2)) {
			suggestions.push(suggest);
		} else {
			suggestions.push(typo.to_string());
		}
	}
	suggestions.join(" ")
}

pub fn best_match_path(typo: &str) -> Option<String> {
	let path_files = get_path_files();
	find_similar(typo, &path_files, Some(3))
}

// higher the threshold, the stricter the comparison
// 1: anything
// 2: 50%
// 3: 33%
// ... etc
pub fn find_similar(typo: &str, candidates: &[String], threshold: Option<usize>) -> Option<String> {
	let threshold = threshold.unwrap_or(2);
	let mut min_distance = typo.chars().count() / threshold + 1;
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
	}
	if let Some(min_distance_index) = min_distance_index {
		return Some(candidates[min_distance_index].to_string());
	}
	None
}

#[allow(clippy::needless_range_loop)]
pub fn compare_string(a: &str, b: &str) -> usize {
	let mut matrix = vec![vec![0; b.chars().count() + 1]; a.chars().count() + 1];

	for i in 0..a.chars().count() + 1 {
		matrix[i][0] = i;
	}
	for j in 0..b.chars().count() + 1 {
		matrix[0][j] = j;
	}

	for (i, ca) in a.chars().enumerate() {
		for (j, cb) in b.chars().enumerate() {
			let cost = if ca == cb { 0 } else { 1 };
			matrix[i + 1][j + 1] = std::cmp::min(
				std::cmp::min(matrix[i][j + 1] + 1, matrix[i + 1][j] + 1),
				matrix[i][j] + cost,
			);
		}
	}
	matrix[a.chars().count()][b.chars().count()]
}

pub fn confirm_suggestion(data: &Data) -> Result<(), String> {
	let shell = &data.shell;
	let command = &data.suggest.clone().unwrap();
	#[cfg(debug_assertions)]
	eprintln!("running command: {command}");

	let now = Instant::now();
	let process = run_suggestion(data, command);

	if process.success() {
		let cd = shell_evaluated_commands(shell, command);
		if let Some(cd) = cd {
			println!("{}", cd);
		}
		Ok(())
	} else {
		if now.elapsed() > Duration::from_secs(3) {
			exit(1);
		}
		suggestion_err(data, command)
	}
}

fn run_suggestion(data: &Data, command: &str) -> std::process::ExitStatus {
	let shell = &data.shell;
	let privilege = &data.privilege;
	match privilege {
		Some(sudo) => std::process::Command::new(sudo)
			.arg(shell)
			.arg("-c")
			.arg(command)
			.stdout(stderr())
			.stderr(Stdio::inherit())
			.spawn()
			.expect("failed to execute process")
			.wait()
			.unwrap(),
		None => std::process::Command::new(shell)
			.arg("-c")
			.arg(command)
			.stdout(stderr())
			.stderr(Stdio::inherit())
			.spawn()
			.expect("failed to execute process")
			.wait()
			.unwrap(),
	}
}

fn suggestion_err(data: &Data, command: &str) -> Result<(), String> {
	let shell = &data.shell;
	let privilege = &data.privilege;
	let process = match privilege {
		Some(sudo) => std::process::Command::new(sudo)
			.arg(shell)
			.arg("-c")
			.arg(command)
			.env("LC_ALL", "C")
			.output()
			.expect("failed to execute process"),
		None => std::process::Command::new(shell)
			.arg("-c")
			.arg(command)
			.env("LC_ALL", "C")
			.output()
			.expect("failed to execute process"),
	};
	let error_msg = match process.stderr.is_empty() {
		true => String::from_utf8_lossy(&process.stdout).to_lowercase(),
		false => String::from_utf8_lossy(&process.stderr).to_lowercase(),
	};
	Err(error_msg.to_string())
}
