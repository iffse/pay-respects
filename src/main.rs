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

fn main() {
	args::handle_args();

	let shell = match std::env::var("_PR_SHELL") {
		Ok(shell) => shell,
		Err(_) => {
			eprintln!("No _PR_SHELL in environment. Did you aliased the command with the correct argument?\n\nUse `pay-respects -h` for help");
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
					let msg = execution.err().unwrap();
					error_msg = msg.to_lowercase();

					let retry_message =
						format!("{}", "Looking for new suggestion...".cyan().bold());

					// println!("\n{} {}", "ERROR:".red().bold(), msg);
					println!("\n{}\n", retry_message);
				}
			} else {
				break;
			}
		} else {
			break;
		}
	}
	println!(
		"No correction found for the command: {}\n",
		last_command.red()
	);
	println!(
		"If you think there should be a correction, please open an issue or send a pull request!"
	);
}
