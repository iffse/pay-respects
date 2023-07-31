use colored::*;
use crate::corrections::split_command;

pub fn highlight_difference(corrected_command: &str, last_command: &str) -> String {
	let mut highlighted_command = String::new();

	let split_corrected_command = split_command(corrected_command);
	let split_last_command = split_command(last_command);

	for new in split_corrected_command {
		if new == "" {
			continue;
		}
		let mut changed = true;
		for old in split_last_command.clone() {
			if new == old {
				changed = false;
				break;
			}
		}
		if changed {
			let colored = new.red().bold();
			highlighted_command = format!("{}{}", highlighted_command, colored);
		} else {
			let colored = new.green();
			highlighted_command = format!("{}{}", highlighted_command, colored);
		}
		highlighted_command.push(' ');
	}

	highlighted_command.trim().to_string()
}
