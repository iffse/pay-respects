use colored::Colorize;
use crate::style::highlight_difference;

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
	let last_command = shell::last_command_expanded_alias(&shell);
	let corrected_command = suggestions::suggest_command(&shell, &last_command);

	if let Some(corrected_command) = corrected_command {
		let command_difference = highlight_difference(&shell, &corrected_command, &last_command);
		if let Some(highlighted_command) = command_difference {
			suggestions::confirm_suggestion(&shell, &corrected_command, &highlighted_command);
			return;
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
