// pay-respects: Press F to correct your command
// Copyright (C) 2023 iff

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

use crate::{shell::command_output, style::highlight_difference};
use colored::Colorize;

mod args;
mod files;
mod shell;
mod style;
mod suggestions;

#[macro_use]
extern crate rust_i18n;
i18n!("i18n", fallback = "en", minify_key = true);

fn main() {
	colored::control::set_override(true);

	args::handle_args();

	let shell = match std::env::var("_PR_SHELL") {
		Ok(shell) => shell,
		Err(_) => {
			eprintln!(
				"{}",
				t!("no-env-setup", var = "_PR_SHELL", help = "pay-respects -h")
			);
			std::process::exit(1);
		}
	};

	let mut last_command = shell::last_command_expanded_alias(&shell);
	let mut error_msg = command_output(&shell, &last_command);
	loop {
		let corrected_command = suggestions::suggest_command(&shell, &last_command, &error_msg);

		if let Some(corrected_command) = corrected_command {
			let command_difference =
				highlight_difference(&shell, &corrected_command, &last_command);
			if let Some(highlighted_command) = command_difference {
				let execution = suggestions::confirm_suggestion(
					&shell,
					&corrected_command,
					&highlighted_command,
				);
				if execution.is_ok() {
					return;
				} else {
					last_command = corrected_command;
					error_msg = execution.err().unwrap();

					let retry_message = format!("{}...", t!("retry"));

					// println!("\n{} {}", "ERROR:".red().bold(), msg);
					eprintln!("\n{}\n", retry_message.cyan().bold());
				}
			} else {
				break;
			}
		} else {
			break;
		}
	}
	eprintln!("{}: {}\n", t!("no-suggestion"), last_command.red());
	eprintln!("{}\n{}", t!("contribute"), "https://github.com/iffse/pay-respects");
}
