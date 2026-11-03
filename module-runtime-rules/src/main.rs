// pay-respects-runtime-module: Runtime parsing of rules
// Copyright (C) 2024 iff

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod config;
mod loader;
mod script;

use std::path::Path;

use pay_respects_rule_engine::{Context, Mode as RuleMode, NoFunctions, evaluate_rule};
use pay_respects_utils::{
	evals::{split_command, split_comment},
	files::get_path_files,
	modes::{Mode, run_mode},
	shell::{detect_shell_type, normalize_shell_type},
};

use crate::config::{get_target_rule, load_config};
use crate::loader::{RuleLoader, script_dirs};
use crate::script::ExternalScripts;

fn main() -> Result<(), std::io::Error> {
	let executable = std::env::var("_PR_COMMAND").expect("_PR_COMMAND not set");
	let shell = std::env::var("_PR_SHELL").expect("_PR_SHELL not set");
	let shell_type = std::env::var("_PR_SHELL_TYPE")
		.ok()
		.filter(|shell_type| !shell_type.is_empty())
		.and_then(|shell_type| normalize_shell_type(&shell_type).map(str::to_string))
		.unwrap_or_else(|| {
			detect_shell_type(&shell)
				.map(str::to_string)
				.unwrap_or_else(|| shell.clone())
		});
	let mut last_command = std::env::var("_PR_LAST_COMMAND").expect("_PR_LAST_COMMAND not set");
	let error_msg = std::env::var("_PR_ERROR_MSG").expect("_PR_ERROR_MSG not set");

	#[cfg(debug_assertions)]
	{
		eprintln!("shell: {}", shell);
		eprintln!("shell_type: {}", shell_type);
		eprintln!("executable: {}", executable);
		eprintln!("last_command: {}", last_command);
		eprintln!("error_msg: {}", error_msg);
	}

	let mut split = split_command(&last_command);
	if split_comment(&mut split).is_some() {
		last_command = split.join(" ");
	}

	pay_respects_utils::shell::set_shell_type(&shell_type);
	pay_respects_utils::settings::load_config();

	let config = load_config();
	let target_rule = get_target_rule(&executable, &config);

	let mut loader = RuleLoader::new();
	let mut rules = Vec::new();

	if let Some(loaded) = loader.load(&target_rule) {
		rules.push(loaded.clone());
	}
	if !target_rule.eq_ignore_ascii_case("_PR_GENERAL")
		&& let Some(loaded) = loader.load("_PR_GENERAL")
	{
		rules.push(loaded.clone());
	}

	let executables = if rules.iter().any(|rule| rule.rule.needs_executables()) {
		load_executables()
	} else {
		Vec::new()
	};
	let script_dirs = script_dirs();

	let mode = match run_mode() {
		Mode::Inline => RuleMode::Inline,
		_ => RuleMode::Suggestion,
	};
	let context = Context {
		shell: &shell,
		shell_type: &shell_type,
		last_command: &last_command,
		error_msg: &error_msg,
		executables: &executables,
	};

	let mut functions = NoFunctions;
	let mut seen = Vec::new();
	let command = last_command.trim();
	for loaded in &rules {
		let rule_dir = loaded.path.parent().unwrap_or(Path::new("."));
		let mut scripts = ExternalScripts::new(rule_dir, &script_dirs);
		for suggestion in evaluate_rule(&loaded.rule, &context, mode, &mut functions, &mut scripts)
		{
			let suggestion = suggestion.trim();
			if suggestion.is_empty()
				|| suggestion == command
				|| seen.iter().any(|item: &String| item == suggestion)
			{
				continue;
			}
			print!("{}", suggestion);
			print!("<_PR_BR>");
			seen.push(suggestion.to_string());
		}
	}

	Ok(())
}

fn load_executables() -> Vec<String> {
	if let Ok(executables) = std::env::var("_PR_EXECUTABLES")
		&& !executables.is_empty()
	{
		return executables
			.split(' ')
			.map(str::to_string)
			.collect::<Vec<String>>();
	}

	get_path_files()
}
