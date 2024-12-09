use std::io::stderr;
use std::process::{exit, Stdio};
use std::time::{Duration, Instant};

use colored::Colorize;
use inquire::*;

use crate::rules::match_pattern;
use crate::shell::{add_candidates_no_dup, module_output, shell_evaluated_commands, Data};
use crate::style::highlight_difference;

pub fn suggest_candidates(data: &mut Data) {
	let executable = &data.split[0].rsplit(std::path::MAIN_SEPARATOR).next().unwrap();
	let command = &data.command;
	let privilege = &data.privilege;
	let mut suggest_candidates = vec![];

	let modules = &data.modules;
	let fallbacks = &data.fallbacks;

	if privilege.is_none() {
		if let Some(candidates) = match_pattern("_PR_privilege", data) {
			add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
		}
	}
	if let Some(candidates) = match_pattern(executable, data) {
		add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
	}
	if let Some(candidates) = match_pattern("_PR_general", data) {
		add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
	}

	#[cfg(debug_assertions)]
	{
		eprintln!("modules: {modules:?}");
		eprintln!("fallbacks: {fallbacks:?}");
	}

	for module in modules {
		let new_candidates = module_output(data, module);

		if let Some(candidates) = new_candidates {
			add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
		}
	}

	if !suggest_candidates.is_empty() {
		data.candidates = suggest_candidates;
		return;
	}
	for fallback in fallbacks {
		let candidates = module_output(data, fallback);
		if candidates.is_some() {
			add_candidates_no_dup(command, &mut suggest_candidates, &candidates.unwrap());
			data.candidates = suggest_candidates;
			return;
		}
	}
}

pub fn select_candidate(data: &mut Data) {
	let candidates = &data.candidates;
	#[cfg(debug_assertions)]
	eprintln!("candidates: {candidates:?}");
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
		let hint = format!("{} {} {}", "[↑/↓]".blue(), confirm, "[Ctrl+C]".red());
		eprintln!("{}", msg);
		eprintln!("{}", hint);

		let ans = Select::new("\n", highlight_candidates.clone())
			.with_page_size(1)
			.with_vim_mode(true)
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

pub fn run_suggestion(data: &Data, command: &str) -> std::process::ExitStatus {
	let shell = &data.shell;
	let privilege = &data.privilege;
	match privilege {
		Some(sudo) => std::process::Command::new(sudo)
			.arg(shell)
			.arg("-c")
			.arg(command)
			.stdout(stderr())
			.stderr(Stdio::inherit())
			.status()
			.expect("failed to execute process"),
		None => std::process::Command::new(shell)
			.arg("-c")
			.arg(command)
			.stdout(stderr())
			.stderr(Stdio::inherit())
			.status()
			.expect("failed to execute process"),
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
