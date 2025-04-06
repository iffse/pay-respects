// pay-respects-ai-module: Request AI suggestions for command errors
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

use crate::requests::ai_suggestion;
use colored::Colorize;
use textwrap::fill;
mod requests;

#[macro_use]
extern crate rust_i18n;
i18n!("i18n", fallback = "en", minify_key = true);

fn main() -> Result<(), std::io::Error> {
	let mode = std::env::var("_PR_MODE");
	if let Ok(mode) = mode {
		if mode.as_str() == "noconfirm" {
			return Ok(());
		}
	}

	let command = std::env::var("_PR_LAST_COMMAND").expect("_PR_LAST_COMMAND not set");
	let error = std::env::var("_PR_ERROR_MSG").expect("_PR_ERROR_MSG not set");
	colored::control::set_override(true);

	#[cfg(debug_assertions)]
	{
		eprintln!("last_command: {}", command);
		eprintln!("error_msg: {}", error);
	}

	// skip for commands with no arguments,
	// very likely to be an error showing the usage
	if command.split_whitespace().count() == 1 {
		return Ok(());
	}
	let suggest = ai_suggestion(&command, &error);
	if let Some(suggest) = suggest {
		if let Some(thinking) = suggest.think {
			let note = format!("{}:", t!("ai-thinking")).bold().blue();
			let thinking = fill(&thinking, termwidth());
			eprintln!("{}{}", note, thinking);
		}
		let warn = format!("{}:", t!("ai-suggestion")).bold().blue();
		let note = fill(&suggest.suggestion.note, termwidth());

		eprintln!("{}\n{}\n", warn, note);
		let suggestions = suggest.suggestion.commands;
		for suggestion in suggestions {
			print!("{}<_PR_BR>", suggestion);
		}
	}
	Ok(())
}

fn termwidth() -> usize {
	use terminal_size::{terminal_size, Height, Width};
	let size = terminal_size();
	if let Some((Width(w), Height(_))) = size {
		std::cmp::min(w as usize, 80)
	} else {
		80
	}
}
