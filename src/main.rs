use colored::Colorize;

mod args;
mod corrections;
mod shell;
mod style;

fn main() {
	std::env::set_var("LC_ALL", "C");
	args::handle_args();

	let shell = std::env::var("_PR_SHELL").expect(
		"No _PR_SHELL in environment. Did you aliased the binary with the correct arguments?",
	);
	let last_command = shell::last_command_expanded_alias(&shell);
	let corrected_command = corrections::correct_command(&shell, &last_command);

	if let Some(corrected_command) = corrected_command {
		if corrected_command != last_command {
			corrections::confirm_correction(&shell, &corrected_command, &last_command);
			return;
		}
	}

	println!(
		"No correction found for the command: {}\n",
		last_command.red().bold()
	);
	println!("If you think there should be a correction, please open an issue or send a pull request!");
}
