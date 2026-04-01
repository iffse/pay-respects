// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use colored::*;

const PROJECT_NAME: &str = env!("CARGO_PKG_NAME");

pub fn print_warning(message: &str) {
	eprintln!("{}: {}", PROJECT_NAME.yellow(), message);
}

pub fn print_error(message: &str) {
	eprintln!("{}: {}", PROJECT_NAME.red().bold(), message);
}

pub fn unexpected_format(message: &str) {
	print_error(&format!("Unexpected format: {}", message));
}
