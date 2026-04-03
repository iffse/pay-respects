// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::macros::*;

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

pub fn shell_path_post_processing(path: &str) -> String {
	let current_directory_prefix = format!(".{}", std::path::MAIN_SEPARATOR);
	let mut path = if path.starts_with(&current_directory_prefix) {
		path.replacen(&current_directory_prefix, "", 1)
	} else {
		path.to_string()
	};

	if path.contains(' ') {
		path = format!("\"{}\"", path);
		// match get_shell_type() {
		// 	Nu => {
		// 		*path = format!("`{}`", path);
		// 	}
		// 	_ => {
		// 		*path = format!("\"{}\"", path);
		// 	}
		// }
	}

	path.to_string()

	// if let Nu = get_shell_type() {
	// let formatted_path = format!("`{}`", path.replace("\\ ", " "));
	// using regex instead to avoid escaped backslashes
	// let re = Regex::new(r"(?<!\\)\\ ").unwrap();
	// let formatted_path = format!("`{}`", re.replace_all(path, "\\ "));
	// ^^^ Unsupported by regex carte

	// 	let formatted_path = replace_escaped_character(path, ' ', " ");
	// 	let formatted_path = if formatted_path.contains(' ') {
	// 		format!("`{}`", formatted_path)
	// 	} else {
	// 		formatted_path
	// 	};
	// 	*path = formatted_path;
	// }
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
