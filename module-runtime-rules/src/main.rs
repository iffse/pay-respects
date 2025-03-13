// pay-respects-runtime-module: Runtime parsing of rules
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

mod replaces;
mod rules;
use pay_respects_utils::files::get_path_files;

fn main() -> Result<(), std::io::Error> {
	let executable = std::env::var("_PR_COMMAND").expect("_PR_COMMAND not set");
	let shell = std::env::var("_PR_SHELL").expect("_PR_SHELL not set");
	let last_command = std::env::var("_PR_LAST_COMMAND").expect("_PR_LAST_COMMAND not set");
	let error_msg = std::env::var("_PR_ERROR_MSG").expect("_PR_ERROR_MSG not set");
	let executables: Vec<String> = get_path_files();

	#[cfg(debug_assertions)]
	{
		eprintln!("shell: {}", shell);
		eprintln!("executable: {}", executable);
		eprintln!("last_command: {}", last_command);
		eprintln!("error_msg: {}", error_msg);
		eprintln!("executables: {:?}", executables);
	}

	rules::runtime_match(&executable, &shell, &last_command, &error_msg, &executables);
	Ok(())
}
