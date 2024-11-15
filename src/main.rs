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
use sys_locale::get_locale;

mod args;
mod files;
mod rules;
mod shell;
mod style;
mod suggestions;

#[cfg(feature = "runtime-rules")]
mod replaces;
#[cfg(feature = "runtime-rules")]
mod runtime_rules;

#[macro_use]
extern crate rust_i18n;
i18n!("i18n", fallback = "en", minify_key = true);

fn main() {
	colored::control::set_override(true);
	// let locale = std::env::var("LANG").unwrap_or("en_US".to_string());
	let locale = get_locale().unwrap_or("en_US".to_string());
	rust_i18n::set_locale(&locale[0..2]);

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

	let mut last_command = shell::last_command(&shell);
	last_command = shell::expand_alias(&shell, &last_command);
	let mut error_msg = command_output(&shell, &last_command);
	loop {
		let suggestion = {
			let command = suggestions::suggest_command(&shell, &last_command, &error_msg);
			if command.is_none() {
				break;
			};
			command.unwrap()
		};

		let highlighted_suggestion = {
			let difference = highlight_difference(&shell, &suggestion, &last_command);
			if difference.is_none() {
				break;
			};
			difference.unwrap()
		};

		let execution =
			suggestions::confirm_suggestion(&shell, &suggestion, &highlighted_suggestion);
		if execution.is_ok() {
			return;
		} else {
			last_command = suggestion;
			error_msg = execution.err().unwrap();

			let retry_message = format!("{}...", t!("retry"));

			eprintln!("\n{}\n", retry_message.cyan().bold());
		}
	}
	eprintln!("{}: {}\n", t!("no-suggestion"), last_command.red());
	eprintln!(
		"{}\n{}",
		t!("contribute"),
		"https://github.com/iffse/pay-respects"
	);
}
