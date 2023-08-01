use crate::suggestions::split_command;
use colored::*;

pub fn highlight_difference(suggested_command: &str, last_command: &str) -> String {
	let split_suggested_command = split_command(suggested_command);
	let split_last_command = split_command(last_command);

	let mut old_entries = Vec::new();
	for command in &split_suggested_command {
		if command.is_empty() {
			continue;
		}
		'old: for old in split_last_command.clone() {
			if command == &old {
				old_entries.push(command.clone());
				break 'old;
			}
		}
	}

	let mut highlighted = suggested_command.to_string();
	for entry in &split_suggested_command {
		if old_entries.contains(entry) {
			highlighted = highlighted.replace(entry, &entry.cyan().to_string());
		} else {
			highlighted = highlighted.replace(entry, &entry.red().bold().to_string());
		}
	}

	highlighted
}
