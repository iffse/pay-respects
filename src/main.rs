use crate::{shell::command_output, style::highlight_difference};
use colored::Colorize;

mod args;
mod files;
mod shell;
mod style;
mod suggestions;

fn main() {
	args::handle_args();

	let shell = std::env::var("_PR_SHELL").expect(
		"No _PR_SHELL in environment. Did you aliased the binary with the correct arguments?",
	);
	let mut last_command = shell::last_command_expanded_alias(&shell);
	let mut error_msg = command_output(&shell, &last_command);
	loop {
		let corrected_command = suggestions::suggest_command(&shell, &last_command, &error_msg);

		if let Some(corrected_command) = corrected_command {
			let command_difference =
				highlight_difference(&shell, &corrected_command, &last_command);
			if let Some(highlighted_command) = command_difference {
				let execution = suggestions::confirm_suggestion(
					&shell,
					&corrected_command,
					&highlighted_command,
				);
				if execution.is_ok() {
					return;
				} else {
					let retry_message =
						format!("{}", "Looking for new suggestion...".cyan().bold());
					println!("\n{}\n", retry_message);
					last_command = corrected_command;
					error_msg = execution.err().unwrap();
				}
			} else {
				break;
			}
		} else {
			break;
		}
	}
	println!(
		"No correction found for the command: {}\n",
		last_command.red()
	);
	println!(
		"If you think there should be a correction, please open an issue or send a pull request!"
	);
}
