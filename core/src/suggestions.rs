use std::io::stderr;
use std::process::{Stdio, exit};
use std::thread;
use std::time::{Duration, Instant};

use colored::Colorize;
use pay_respects_select::select;
use pay_respects_utils::strings::print_error;

use crate::config;
use crate::data::Data;
use crate::rules::match_pattern;
use crate::shell::{
	add_candidates_no_dup, add_privilege, module_output, shell_evaluated_commands, shell_syntax,
};
use crate::style::highlight_difference;

pub fn suggest_candidates(data: &mut Data) {
	if data.split.is_empty() {
		return;
	}
	let shell = &data.shell;
	let target_rule = data.get_target_rule();
	let command = &data.command;
	let privilege = &data.privilege;
	let mut suggest_candidates = vec![];
	let mut module_candidates = vec![];
	let mut final_candidates = vec![];

	let modules = &data.modules;
	let fallbacks = &data.fallbacks;

	#[cfg(debug_assertions)]
	{
		eprintln!("/// suggest_candidates");
		eprintln!("split: {:?}", data.split);
		eprintln!("command: {command}");
		eprintln!("privilege: {privilege:?}");
		eprintln!("modules: {modules:?}");
		eprintln!("fallbacks: {fallbacks:?}");
	}

	thread::scope(|s| {
		s.spawn(|| {
			for module in modules {
				let new_candidates = module_output(data, module);

				if let Some(candidates) = new_candidates {
					add_candidates_no_dup(command, &mut module_candidates, &candidates);
				}
			}
		});

		if let Some(candidates) = match_pattern(target_rule, data) {
			add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
		}
		if let Some(candidates) = match_pattern("_PR_general", data) {
			add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
		}
		if privilege.is_none()
			&& let Some(candidates) = match_pattern("_PR_privilege", data)
		{
			add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
		}
	});

	if !module_candidates.is_empty() {
		add_candidates_no_dup(command, &mut final_candidates, &module_candidates);
	}
	if !suggest_candidates.is_empty() {
		add_candidates_no_dup(command, &mut final_candidates, &suggest_candidates);
	}

	if !final_candidates.is_empty() {
		data.candidates = final_candidates
			.iter()
			.map(|s| shell_syntax(shell, s))
			.collect();
		return;
	}
	for fallback in fallbacks {
		let candidates = module_output(data, fallback);
		if let Some(candidates) = candidates {
			add_candidates_no_dup(command, &mut final_candidates, &candidates);
			data.candidates = final_candidates
				.iter()
				.map(|s| shell_syntax(shell, s))
				.collect();
			return;
		}
	}
}

pub fn select_candidate(data: &mut Data) {
	let candidates = &data.candidates;
	#[cfg(debug_assertions)]
	eprintln!("candidates: {candidates:?}");

	let mut active_candidates = candidates
		.iter()
		.map(|candidate| highlight_difference(data, candidate, true).unwrap())
		.collect::<Vec<String>>();

	if active_candidates.iter().any(|x| x.contains('\n')) {
		for candidate in active_candidates.iter_mut() {
			*candidate = candidate.replace("\n", "\r\n     ").to_string();
		}
	}

	let mut inactive_candidates = candidates
		.iter()
		.map(|candidate| highlight_difference(data, candidate, false).unwrap())
		.collect::<Vec<String>>();

	if inactive_candidates.iter().any(|x| x.contains('\n')) {
		for candidate in inactive_candidates.iter_mut() {
			*candidate = candidate.replace("\n", "\r\n     ").to_string();
		}
	}

	let msg = format!("{}", t!("multi-suggest", num = candidates.len()))
		.bold()
		.blue();
	let confirm = format!("[{}]", t!("confirm-yes")).green();
	let hint = format!("{} {} {}", "[↑/↓/j/k]".blue(), confirm, "[ESC]".red());
	let prelude = format!("{}\n\r{}", msg, hint);

	let selection =
		select(&prelude, &active_candidates, &inactive_candidates).unwrap_or_else(|err| {
			print_error(&format!("Selection failed: {}", err));
			exit(1);
		});

	let suggestion = candidates[selection].to_string();
	data.update_suggest(&suggestion);
	data.expand_suggest();

	data.candidates.clear();
}

pub fn confirm_suggestion(data: &Data) -> Result<(), String> {
	let shell = &data.shell;
	let command = &data.suggest.clone().unwrap();
	#[cfg(debug_assertions)]
	eprintln!("running command: {command}");

	let eval_method = &data.config.eval_method;
	if eval_method == &config::EvalMethod::Shell {
		shell_suggestion(data, command);
		return Ok(());
	};

	let now = Instant::now();
	let process = run_suggestion(data, command);

	if process.success() {
		shell_evaluated_commands(shell, command, true);
		Ok(())
	} else {
		shell_evaluated_commands(shell, command, false);
		if now.elapsed() > Duration::from_secs(3) {
			exit(1);
		}
		suggestion_err(data, command)
	}
}

pub fn inline_suggestion(data: &mut Data) {
	if data.split.is_empty() {
		return;
	}
	let shell = &data.shell;
	let target_rule = data.get_target_rule();
	let command = &data.command;

	let mut suggest_candidates = vec![];

	if let Some(candidates) = match_pattern(target_rule, data) {
		add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
	}
	if let Some(candidates) = match_pattern("_PR_general", data) {
		add_candidates_no_dup(command, &mut suggest_candidates, &candidates);
	}

	if !suggest_candidates.is_empty() {
		data.candidates = suggest_candidates
			.iter()
			.map(|s| shell_syntax(shell, s))
			.collect();
	}
}

pub fn run_suggestion(data: &Data, command: &str) -> std::process::ExitStatus {
	let shell = &data.shell;
	let privilege = &data.privilege;
	let command = if let Some(env) = &data.env {
		format!("{env} {command}")
	} else {
		command.to_string()
	};
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

pub fn shell_suggestion(data: &Data, command: &str) {
	let shell = &data.shell;
	let privilege = &data.privilege;
	let command = if let Some(env) = &data.env {
		format!("{env} {command}")
	} else {
		command.to_string()
	};

	let command = if let Some(privilege) = privilege {
		add_privilege(shell, privilege, &command)
	} else {
		command
	};
	println!("{}", command);
}

fn suggestion_err(data: &Data, command: &str) -> Result<(), String> {
	let shell = &data.shell;
	let privilege = &data.privilege;
	let command = if let Some(env) = &data.env {
		format!("{env} {command}")
	} else {
		command.to_string()
	};
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
