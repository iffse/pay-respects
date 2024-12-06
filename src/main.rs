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

use sys_locale::get_locale;

mod args;
mod files;
mod modes;
mod rules;
mod shell;
mod style;
mod suggestions;
mod system;

#[cfg(feature = "runtime-rules")]
mod replaces;
#[cfg(feature = "runtime-rules")]
mod runtime_rules;

#[cfg(feature = "request-ai")]
mod requests;

#[macro_use]
extern crate rust_i18n;
i18n!("i18n", fallback = "en", minify_key = true);

fn main() -> Result<(), std::io::Error>{
	colored::control::set_override(true);
	let mut data = {
		let init = init();
		if init.is_none() {
			return Ok(());
		} else {
			init.unwrap()
		}
	};

	data.expand_command();
	use shell::Mode;
	match data.mode {
		Mode::Suggestion => modes::suggestion(&mut data),
		Mode::CNF => modes::cnf(&mut data),
	}

	Ok(())
}

fn init() -> Option<shell::Data> {
	let locale = {
		let sys_locale = get_locale().unwrap_or("en-US".to_string());
		if sys_locale.len() < 2 {
			"en_US".to_string()
		} else {
			sys_locale
		}
	};
	rust_i18n::set_locale(&locale[0..2]);

	let exit = args::handle_args();
	if exit {
		return None;
	}

	#[cfg(feature = "request-ai")]
	{
		if std::env::var("_PR_AI_LOCALE").is_err() {
			std::env::set_var("_PR_AI_LOCALE", &locale);
		}
	}

	Some(shell::Data::init())
}
