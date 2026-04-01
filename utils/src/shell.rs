// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::macros::*;
use crate::strings::replace_escaped_character;

#[derive(Debug)]
pub enum ShellType {
	Generic,
	Bash,
	Zsh,
	Fish,
	Nu,
	Powershell,
}

use ShellType::*;

pub static mut SHELL_TYPE: ShellType = Generic;

/// Modifies the given path string to be in the correct format for the specified shell type.
///
/// Currently only used by nushell that does not allow `\ `
pub fn shell_path_style(path: &mut String) {
	if let Nu = get_shell_type() {
		// let formatted_path = format!("`{}`", path.replace("\\ ", " "));
		// using regex instead to avoid escaped backslashes
		// let re = Regex::new(r"(?<!\\)\\ ").unwrap();
		// let formatted_path = format!("`{}`", re.replace_all(path, "\\ "));
		// ^^^ Unsupported by regex carte

		let formatted_path = replace_escaped_character(path, ' ', " ").replace("\\\\", "\\");
		let formatted_path = if formatted_path.contains(' ') {
			format!("`{}`", formatted_path)
		} else {
			formatted_path
		};
		*path = formatted_path;
	}
}

pub fn reverse_shell_path_style(path: &mut String) {
	if let Nu = get_shell_type() {
		let formatted_path = path
			.trim_matches('`')
			.replace("\\", "\\\\")
			.replace(" ", "\\ ");
		*path = formatted_path;
	}
}

pub fn set_shell_type(shell: &str) {
	match shell {
		"bash" => static_write!(SHELL_TYPE, Bash),
		"zsh" => static_write!(SHELL_TYPE, Zsh),
		"fish" => static_write!(SHELL_TYPE, Fish),
		"nu" | "nushell" => static_write!(SHELL_TYPE, Nu),
		"powershell" | "pwsh" => static_write!(SHELL_TYPE, Powershell),
		_ => {}
	}
}

pub fn get_shell_type() -> ShellType {
	static_read!(SHELL_TYPE)
}
