mod corrections;
mod shell;
mod style;

use shell::get_privilege;

fn main() {
	std::env::set_var("LC_ALL", "C");

	let shell = shell::find_shell();
	let last_command = shell::find_last_command(&shell);
	let corrected_command = corrections::correct_command(&shell, &last_command);

	if let Some(mut corrected_command) = corrected_command {
		if corrected_command.starts_with("sudo ") {
			let privilege = get_privilege();
			if let Some(privilege) = privilege {
				if privilege != "sudo" {
					corrected_command = corrected_command.replacen("sudo", &privilege, 1);
				}
			}
		}
		corrections::confirm_correction(&shell, &corrected_command, &last_command);
	} else {
		println!(
			"
No correction found.

If you think there should be a correction, please open an issue or send a pull request!"
		);
	}
}
