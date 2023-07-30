mod corrections;
mod shell;
mod style;

fn main() {
	std::env::set_var("LC_ALL", "C");

	let shell = shell::find_shell();
	let last_command = shell::find_last_command(&shell);
	let corrected_command = corrections::correct_command(&shell, &last_command);
	if let Some(corrected_command) = corrected_command {
		corrections::confirm_correction(&shell, &corrected_command, &last_command);
	} else {
		println!("No correction found.");
	}
}
