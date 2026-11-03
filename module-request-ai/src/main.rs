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

use std::env;
use std::io::IsTerminal;

use colored::Colorize;
use pay_respects_select::select_simple;
use pay_respects_utils::strings::print_error;
use sys_locale::get_locale;

use crate::requests::ai_suggestion;
mod buffer;
mod config;
mod requests;

#[macro_use]
extern crate rust_i18n;
i18n!("i18n", fallback = "en", minify_key = true);

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
	if std::env::var("_PR_AI_DISABLE").is_ok() {
		return Ok(());
	}
	let mode = std::env::var("_PR_MODE").ok();
	if mode.as_deref() == Some("noconfirm") {
		return Ok(());
	}

	let locale = {
		let sys_locale = {
			// use terminal locale if available
			if let Ok(locale) = env::var("LANG") {
				locale
			} else if let Ok(locale) = env::var("LC_ALL") {
				locale
			} else if let Ok(locale) = env::var("LC_MESSAGES") {
				locale
			} else {
				get_locale().unwrap_or("en-US".to_string())
			}
		};
		if sys_locale.len() < 2 {
			"en-US".to_string()
		} else {
			sys_locale
		}
	};
	rust_i18n::set_locale(&locale[0..2]);
	colored::control::set_override(true);

	let config = config::load();
	if config.enabled == Some(false) {
		return Ok(());
	}

	let command = std::env::var("_PR_LAST_COMMAND").expect("_PR_LAST_COMMAND not set");
	let error = std::env::var("_PR_ERROR_MSG").expect("_PR_ERROR_MSG not set");

	pay_respects_utils::dlog!(2, "last_command: {}\nerror_msg: {}", command, error);

	// skip for commands with no arguments,
	// very likely to be an error showing the usage
	if command.split_whitespace().count() == 1 {
		return Ok(());
	}

	if !ai_enabled(&config, mode.as_deref()) {
		return Ok(());
	}
	ai_suggestion(&command, &error, &locale, &config).await;

	Ok(())
}

fn ai_enabled(config: &config::Config, mode: Option<&str>) -> bool {
	if let Some(enabled) = config.enabled {
		return enabled;
	}

	if config.has_custom_api_key() {
		return true;
	}

	// Do not ask in non-interactive modes or when the prompt would not be visible.
	if matches!(mode, Some("inline") | Some("echo")) || !std::io::stderr().is_terminal() {
		return false;
	}

	prompt_ai_consent()
}

fn prompt_ai_consent() -> bool {
	let msg = t!("ai-consent-prompt").to_string().bold().blue();
	let hint = format!(
		"{} {} {}",
		"[↑/↓/j/k]".blue(),
		"[Enter]".green(),
		"[ESC]".red()
	);
	let prelude = format!("{}\n\r{}", msg, hint);
	let items = [
		t!("ai-consent-builtin").to_string(),
		t!("ai-consent-builtin-always").to_string(),
		t!("ai-consent-disable").to_string(),
	];

	match select_simple(&prelude, &items) {
		Ok(0) => true,
		Ok(1) => {
			if let Err(err) = config::set_enabled(true) {
				print_error(&format!("Failed to save configuration: {}", err));
			}
			true
		}
		Ok(_) => {
			if let Err(err) = config::set_enabled(false) {
				print_error(&format!("Failed to save configuration: {}", err));
			}
			false
		}
		Err(err) => {
			print_error(&format!("Selection failed: {}", err));
			false
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn explicit_enabled_is_respected() {
		let config = config::Config {
			enabled: Some(true),
			..Default::default()
		};
		assert!(ai_enabled(&config, None));
	}

	#[test]
	fn explicit_disabled_is_respected() {
		let config = config::Config {
			enabled: Some(false),
			..Default::default()
		};
		assert!(!ai_enabled(&config, None));
	}
}
