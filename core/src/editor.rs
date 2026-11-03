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

use std::io::Write;
use std::path::Path;
use std::process::ExitStatus;

use pay_respects_utils::strings::print_error;

#[cfg(windows)]
const FALLBACK_EDITOR: &str = "notepad";
#[cfg(not(windows))]
const FALLBACK_EDITOR: &str = "vi";

/// Opens `command` in the user's editor and returns the edited command. `None`
/// when the editor fails, is cancelled, or leaves the file empty.
pub fn edit(command: &str) -> Option<String> {
	edit_with(&editor_command(), command)
}

fn editor_command() -> String {
	for variable in ["VISUAL", "EDITOR"] {
		if let Ok(editor) = std::env::var(variable)
			&& !editor.trim().is_empty()
		{
			return editor;
		}
	}
	FALLBACK_EDITOR.to_string()
}

fn edit_with(editor: &str, command: &str) -> Option<String> {
	let mut file = tempfile::Builder::new()
		.prefix("pay-respects-")
		.suffix(".sh")
		.tempfile()
		.ok()?;
	file.write_all(command.as_bytes()).ok()?;
	file.write_all(b"\n").ok()?;
	file.flush().ok()?;

	if !spawn_editor(editor, file.path())?.success() {
		return None;
	}

	let edited = std::fs::read_to_string(file.path()).ok()?;
	let edited = edited.trim();
	if edited.is_empty() {
		None
	} else {
		Some(edited.to_string())
	}
}

#[cfg(unix)]
fn spawn_editor(editor: &str, path: &Path) -> Option<ExitStatus> {
	let mut command = std::process::Command::new("sh");
	command
		.arg("-c")
		.arg(format!("{} \"$1\"", editor))
		.arg("sh")
		.arg(path);

	// stdout is the command substitution the shell reads the evaluated commands
	// from, so the editor has to talk to the terminal directly to draw anything.
	if let Ok(tty) = std::fs::OpenOptions::new()
		.read(true)
		.write(true)
		.open("/dev/tty")
		&& let (Ok(stdout), Ok(stderr)) = (tty.try_clone(), tty.try_clone())
	{
		command.stdin(tty).stdout(stdout).stderr(stderr);
	}

	match command.status() {
		Ok(status) => Some(status),
		Err(err) => {
			print_error(&format!("Failed to run editor `{}`: {}", editor, err));
			None
		}
	}
}

#[cfg(windows)]
fn spawn_editor(editor: &str, path: &Path) -> Option<ExitStatus> {
	let mut command = std::process::Command::new("cmd");
	command
		.arg("/C")
		.arg(format!("{} \"{}\"", editor, path.display()));

	if let (Ok(stdin), Ok(stdout)) = (
		std::fs::OpenOptions::new().read(true).open("CONIN$"),
		std::fs::OpenOptions::new().write(true).open("CONOUT$"),
	) && let Ok(stderr) = stdout.try_clone()
	{
		command.stdin(stdin).stdout(stdout).stderr(stderr);
	}

	match command.status() {
		Ok(status) => Some(status),
		Err(err) => {
			print_error(&format!("Failed to run editor `{}`: {}", editor, err));
			None
		}
	}
}

#[cfg(all(test, unix))]
mod tests {
	use super::edit_with;
	use std::os::unix::fs::PermissionsExt;

	fn script(content: &str) -> (tempfile::TempDir, String) {
		let dir = tempfile::tempdir().unwrap();
		let path = dir.path().join("editor.sh");
		std::fs::write(&path, content).unwrap();
		std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
		let editor = path.to_string_lossy().to_string();
		(dir, editor)
	}

	#[test]
	fn returns_the_edited_command() {
		let (_dir, editor) = script("#!/bin/sh\nprintf 'git commit\\n' > \"$1\"\n");
		assert_eq!(edit_with(&editor, "gti commit").unwrap(), "git commit");
	}

	#[test]
	fn returns_none_when_the_editor_fails() {
		let (_dir, editor) = script("#!/bin/sh\nexit 1\n");
		assert!(edit_with(&editor, "gti commit").is_none());
	}

	#[test]
	fn returns_none_for_an_empty_command() {
		let (_dir, editor) = script("#!/bin/sh\nprintf '\\n' > \"$1\"\n");
		assert!(edit_with(&editor, "gti commit").is_none());
	}
}
