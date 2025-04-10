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
use sys_locale::get_locale;
mod buffer;
mod requests;

#[macro_use]
extern crate rust_i18n;
i18n!("i18n", fallback = "en", minify_key = true);

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
	if std::env::var("_PR_AI_DISABLE").is_ok() {
		return Ok(());
	}
	let mode = std::env::var("_PR_MODE");
	if let Ok(mode) = mode {
		if mode.as_str() == "noconfirm" {
			return Ok(());
		}
	}

	let locale = {
		let sys_locale = get_locale().unwrap_or("en-US".to_string());
		if sys_locale.len() < 2 {
			"en-US".to_string()
		} else {
			sys_locale
		}
	};
	rust_i18n::set_locale(&locale[0..2]);

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
	ai_suggestion(&command, &error).await;

	Ok(())
}
