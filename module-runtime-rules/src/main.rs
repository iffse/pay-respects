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
mod replaces;
mod rules;
use pay_respects_utils::{
	evals::{split_command, split_comment},
	files::get_path_files,
	modes::{Mode, run_mode},
};

use crate::config::{get_target_rule, load_config};

fn main() -> Result<(), std::io::Error> {
	let executable = std::env::var("_PR_COMMAND").expect("_PR_COMMAND not set");
	let shell = std::env::var("_PR_SHELL").expect("_PR_SHELL not set");
	let mut last_command = std::env::var("_PR_LAST_COMMAND").expect("_PR_LAST_COMMAND not set");
	let error_msg = std::env::var("_PR_ERROR_MSG").expect("_PR_ERROR_MSG not set");
	let executables: Vec<String> = get_path_files();

	let mode = run_mode();
	// unimlemented yet
	if mode == Mode::Inline {
		return Ok(());
	}

	#[cfg(debug_assertions)]
	{
		eprintln!("shell: {}", shell);
		eprintln!("executable: {}", executable);
		eprintln!("last_command: {}", last_command);
		eprintln!("error_msg: {}", error_msg);
		eprintln!("executables: {:?}", executables);
	}

	let mut split = split_command(&last_command);
	if split_comment(&mut split).is_some() {
		last_command = split.join(" ");
	}

	pay_respects_utils::shell::set_shell_type(&shell);
	pay_respects_utils::settings::load_config();

	let config = load_config();
	let target_rule = get_target_rule(&executable, &config);

	let mut runned_rules = vec![];
	let mut pending_rules = vec![target_rule];

	while let Some(executable) = pending_rules.pop() {
		if runned_rules.contains(&executable) {
			continue;
		}
		runned_rules.push(executable.clone());
		let extends =
			rules::runtime_match(&executable, &shell, &last_command, &error_msg, &executables);
		if let Some(extends) = extends {
			for extend in extends {
				if !runned_rules.contains(&extend) {
					pending_rules.push(extend);
				}
			}
		}
	}
	rules::runtime_match(
		"_PR_GENERAL",
		&shell,
		&last_command,
		&error_msg,
		&executables,
	);
	Ok(())
}
