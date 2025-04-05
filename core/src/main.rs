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

use std::env;
use sys_locale::get_locale;

mod args;
mod modes;
mod rules;
mod shell;
mod style;
mod suggestions;
mod system;

#[macro_use]
extern crate rust_i18n;
i18n!("i18n", fallback = "en", minify_key = true);

fn main() -> Result<(), std::io::Error> {
	colored::control::set_override(true);
	let init = init();
	let mut data = if let Err(status) = init {
		match status {
			args::Status::Exit => {
				return Ok(());
			}
			args::Status::Error => {
				return Err(std::io::Error::new(
					std::io::ErrorKind::InvalidInput,
					"Invalid input",
				));
			}
			_ => {
				unreachable!()
			}
		}
	} else {
		init.ok().unwrap()
	};

	use shell::Mode::*;
	match data.mode {
		Suggestion => modes::suggestion(&mut data),
		Echo => modes::echo(&mut data),
		NoConfirm => modes::noconfirm(&mut data),
		Cnf => modes::cnf(&mut data),
	}

	Ok(())
}

fn init() -> Result<shell::Data, args::Status> {
	let locale = {
		let sys_locale = get_locale().unwrap_or("en-US".to_string());
		if sys_locale.len() < 2 {
			"en-US".to_string()
		} else {
			sys_locale
		}
	};
	rust_i18n::set_locale(&locale[0..2]);

	let status = args::handle_args(env::args());
	match status {
		args::Status::Exit => {
			return Err(status);
		}
		args::Status::Error => {
			return Err(status);
		}
		_ => {}
	}

	Ok(shell::Data::init())
}
