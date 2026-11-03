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
	Dash,
	Xonsh,
	Elvish,
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

pub fn normalize_shell_type(shell: &str) -> Option<&'static str> {
	match shell {
		"bash" => Some("bash"),
		"zsh" => Some("zsh"),
		"fish" => Some("fish"),
		"dash" => Some("dash"),
		"xonsh" => Some("xonsh"),
		"elvish" => Some("elvish"),
		"nu" | "nush" | "nushell" => Some("nu"),
		"pwsh" | "powershell" | "ps" => Some("pwsh"),
		_ => None,
	}
}

pub fn detect_shell_type(binary: &str) -> Option<&'static str> {
	let name = std::path::Path::new(binary)
		.file_name()
		.and_then(|name| name.to_str())
		.unwrap_or(binary)
		.to_lowercase();

	if let Some(shell) = normalize_shell_type(&name) {
		return Some(shell);
	}

	for (token, shell) in [
		("powershell", "pwsh"),
		("nushell", "nu"),
		("elvish", "elvish"),
		("xonsh", "xonsh"),
		("dash", "dash"),
		("fish", "fish"),
		("zsh", "zsh"),
		("bash", "bash"),
		("pwsh", "pwsh"),
		("nush", "nu"),
		("nu", "nu"),
	] {
		if contains_shell_token(&name, token) {
			return Some(shell);
		}
	}
	None
}

fn contains_shell_token(name: &str, token: &str) -> bool {
	if name == token {
		return true;
	}

	let mut rest = name;
	while let Some(index) = rest.find(token) {
		let before = rest[..index].chars().next_back();
		let after = rest[index + token.len()..].chars().next();
		if before.is_none_or(|character| !character.is_alphanumeric())
			&& after.is_none_or(|character| !character.is_alphanumeric())
		{
			return true;
		}
		rest = &rest[index + token.len()..];
	}
	false
}

pub fn set_shell_type(shell: &str) {
	match shell {
		"bash" => static_write!(SHELL_TYPE, Bash),
		"zsh" => static_write!(SHELL_TYPE, Zsh),
		"fish" => static_write!(SHELL_TYPE, Fish),
		"dash" => static_write!(SHELL_TYPE, Dash),
		"xonsh" => static_write!(SHELL_TYPE, Xonsh),
		"elvish" => static_write!(SHELL_TYPE, Elvish),
		"nu" | "nushell" => static_write!(SHELL_TYPE, Nu),
		"powershell" | "pwsh" => static_write!(SHELL_TYPE, Powershell),
		_ => {}
	}
}

pub fn get_shell_type() -> ShellType {
	static_read!(SHELL_TYPE)
}

#[cfg(test)]
mod tests {
	use super::{detect_shell_type, normalize_shell_type};

	#[test]
	fn detects_shell_types_from_binary_names() {
		for (binary, shell) in [
			("zsh", "zsh"),
			("zsh-fork", "zsh"),
			("/usr/bin/bash-fork", "bash"),
			("fish-beta", "fish"),
			("xonsh-fork", "xonsh"),
			("elvish-fork", "elvish"),
			("nu", "nu"),
			("nu-fork", "nu"),
			("nushell", "nu"),
			("pwsh-preview", "pwsh"),
			("powershell-fork", "pwsh"),
			("dash-fork", "dash"),
		] {
			assert_eq!(detect_shell_type(binary), Some(shell), "{}", binary);
		}

		assert_eq!(detect_shell_type("dashboard"), None);
		assert_eq!(detect_shell_type("unknown-shell"), None);
	}

	#[test]
	fn normalizes_shell_aliases() {
		assert_eq!(normalize_shell_type("nushell"), Some("nu"));
		assert_eq!(normalize_shell_type("powershell"), Some("pwsh"));
		assert_eq!(normalize_shell_type("zsh-fork"), None);
	}
}
