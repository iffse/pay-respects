use std::io::stderr;
use std::os::fd::AsFd;
use std::process::{exit, Stdio};
use std::time::{Duration, Instant};

use colored::Colorize;
use regex_lite::Regex;

use pay_respects_parser::parse_rules;

use crate::files::{get_best_match_file, get_path_files};
use crate::shell::PRIVILEGE_LIST;

pub fn suggest_command(shell: &str, last_command: &str, error_msg: &str) -> Option<String> {
	let split_command = split_command(last_command);
	let executable = match PRIVILEGE_LIST.contains(&split_command[0].as_str()) {
		true => split_command.get(1).expect(&t!("no-command")).as_str(),
		false => split_command.first().expect(&t!("no-command")).as_str(),
	};

	if !PRIVILEGE_LIST.contains(&executable) {
		let suggest = match_pattern("_PR_privilege", last_command, error_msg, shell);
		if suggest.is_some() {
			return suggest;
		}
	}

	let last_command = match PRIVILEGE_LIST.contains(&split_command[0].as_str()) {
		true => &last_command[split_command[0].len() + 1..],
		false => last_command,
	};

	let suggest = match_pattern(executable, last_command, error_msg, shell);
	if let Some(suggest) = suggest {
		if PRIVILEGE_LIST.contains(&split_command[0].as_str()) {
			return Some(format!("{} {}", split_command[0], suggest));
		}
		return Some(suggest);
	}

	let suggest = match_pattern("_PR_general", last_command, error_msg, shell);
	if let Some(suggest) = suggest {
		if PRIVILEGE_LIST.contains(&split_command[0].as_str()) {
			return Some(format!("{} {}", split_command[0], suggest));
		}
		return Some(suggest);
	}
	None
}

fn match_pattern(
	executable: &str,
	last_command: &str,
	error_msg: &str,
	shell: &str,
) -> Option<String> {
	parse_rules!("rules");
}

fn check_executable(shell: &str, executable: &str) -> bool {
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

fn opt_regex(regex: &str, command: &mut String) -> String {
	let regex = Regex::new(regex).unwrap();
	let opts = regex
		.find_iter(command)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();
	for opt in opts.clone() {
		*command = command.replace(&opt, "");
	}

	opts.join(" ")
}

fn err_regex(regex: &str, error_msg: &str) -> String {
	let regex = Regex::new(regex).unwrap();
	let err = regex
		.find_iter(error_msg)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();

	err.join(" ")
}

fn cmd_regex(regex: &str, command: &str) -> String {
	let regex = Regex::new(regex).unwrap();
	let err = regex
		.find_iter(command)
		.map(|cap| cap.as_str().to_owned())
		.collect::<Vec<String>>();

	err.join(" ")
}

fn eval_shell_command(shell: &str, command: &str) -> Vec<String> {
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

fn suggest_typo(typos: &[String], candidates: &[String]) -> String {
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
					if let Some(suggest) = find_similar(typo, &path_files) {
						suggestions.push(suggest);
					};
				}
				"file" => {
					if let Some(suggest) = get_best_match_file(typo) {
						suggestions.push(suggest);
					}
				}
				_ => {}
			}
		} else if let Some(suggest) = find_similar(typo, candidates) {
			suggestions.push(suggest);
		}
	}
	suggestions.join(" ")
}

pub fn find_similar(typo: &str, candidates: &[String]) -> Option<String> {
	let mut min_distance = 10;
	let mut min_distance_index = None;
	for (i, candidate) in candidates.iter().enumerate() {
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
fn compare_string(a: &str, b: &str) -> usize {
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

pub fn confirm_suggestion(shell: &str, command: &str, highlighted: &str) -> Result<(), String> {
	eprintln!("{}\n", highlighted);
	let confirm = format!("[{}]", t!("confirm-yes")).green();
	eprintln!("{}: {} {}", t!("confirm"), confirm, "[Ctrl+C]".red());
	std::io::stdin().read_line(&mut String::new()).unwrap();

	for p in PRIVILEGE_LIST {
		let _p = p.to_owned() + " ";
		if !command.starts_with(&_p) {
			continue;
		}
		let command = command.replacen(&_p, "", 1);

		let now = Instant::now();
		let process = std::process::Command::new(p)
			.arg(shell)
			.arg("-c")
			.arg(&command)
			// .stdout(Stdio::inherit())
			.stdout(stderr().as_fd().try_clone_to_owned().unwrap())
			.stderr(Stdio::inherit())
			.spawn()
			.expect("failed to execute process")
			.wait()
			.unwrap();

		if process.success() {
			println!("{}", shell_evaluated_commands(&command));
			return Ok(());
		} else {
			if now.elapsed() > Duration::from_secs(3) {
				exit(1);
			}
			let process = std::process::Command::new(p)
				.arg(shell)
				.arg("-c")
				.arg(command)
				.env("LC_ALL", "C")
				.output()
				.expect("failed to execute process");
			let error_msg = match process.stderr.is_empty() {
				true => String::from_utf8_lossy(&process.stdout),
				false => String::from_utf8_lossy(&process.stderr),
			};
			return Err(error_msg.to_string());
		}
	}

	let now = Instant::now();
	let process = std::process::Command::new(shell)
		.arg("-c")
		.arg(command)
		// .stdout(Stdio::inherit())
		.stdout(stderr().as_fd().try_clone_to_owned().unwrap())
		.stderr(Stdio::inherit())
		.spawn()
		.expect("failed to execute process")
		.wait()
		.unwrap();

	if process.success() {
		println!("{}", shell_evaluated_commands(&command));
		Ok(())
	} else {
		if now.elapsed() > Duration::from_secs(3) {
			exit(1);
		}
		let process = std::process::Command::new(shell)
			.arg("-c")
			.arg(command)
			.env("LC_ALL", "C")
			.output()
			.expect("failed to execute process");
		let error_msg = match process.stderr.is_empty() {
			true => String::from_utf8_lossy(&process.stdout).to_lowercase(),
			false => String::from_utf8_lossy(&process.stderr).to_lowercase(),
		};
		Err(error_msg.to_string())
	}
}

fn shell_evaluated_commands(command: &str) -> String {
	let lines = command
		.lines()
		.map(|line| line.trim().trim_end_matches(['\\', ';', '|', '&']))
		.collect::<Vec<&str>>();
	let mut commands = Vec::new();
	for line in lines {
		if line.starts_with("cd ") {
			commands.push(line.to_string());
		}
	}

	commands.join(" && ")
}
