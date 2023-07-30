use colored::*;

pub fn highlight_difference(corrected_command: &str, last_command: &str) -> String {
	let mut highlighted_command = String::new();

	let split_corrected_command = corrected_command.split(' ');
	let split_last_command = last_command.split(' ');

	for new in split_corrected_command {
		let mut changed = true;
		for old in split_last_command.clone() {
			if new == old {
				changed = false;
				break;
			}
		}
		if changed {
			highlighted_command.push_str(&new.red().bold());
		} else {
			highlighted_command.push_str(&new.green());
		}
		highlighted_command.push(' ');
	}

	highlighted_command.pop();
	highlighted_command
}
