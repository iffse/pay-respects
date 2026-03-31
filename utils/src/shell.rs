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

/// Modifies the given path string to be in the correct format for the specified shell type.
///
/// Currently only used by nushell that does not allow `\ `
pub fn shell_path_style(path: &mut String) {
	if let Nu = get_shell_type() {
		let formatted_path = format!("`{}`", path.replace("\\ ", " "));
		*path = formatted_path;
	}
}

pub fn reverse_shell_path_style(path: &mut String) {
	if let Nu = get_shell_type() {
		if path.starts_with('`') && path.ends_with('`') {
			let formatted_path = path.trim_matches('`').replace(" ", "\\ ");
			*path = formatted_path;
		}
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
