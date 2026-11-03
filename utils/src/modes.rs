use std::process::exit;

use crate::strings::print_error;

#[derive(PartialEq, Debug)]
pub enum Mode {
	Suggestion,
	Inline,
	Echo,
	NoConfirm,
	Cnf,
}

pub fn run_mode() -> Mode {
	match std::env::var("_PR_MODE") {
		Ok(mode) => match mode.as_str() {
			"suggest" => Mode::Suggestion,
			"inline" => Mode::Inline,
			"cnf" => Mode::Cnf,
			"noconfirm" => Mode::NoConfirm,
			"echo" => Mode::Echo,
			_ => {
				print_error(&format!("Invalid mode: {}", mode));
				exit(1);
			}
		},
		Err(_) => Mode::Suggestion,
	}
}
