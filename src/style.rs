use crate::shell::PRIVILEGE_LIST;
use crate::suggestions::split_command;
use colored::*;

// to_string() is necessary here, otherwise there won't be color in the output
#[warn(clippy::unnecessary_to_owned)]
pub fn highlight_difference(shell: &str, suggested_command: &str, last_command: &str) -> String {
	let mut split_suggested_command = split_command(suggested_command);
	let split_last_command = split_command(last_command);

	let privileged = PRIVILEGE_LIST.contains(&split_suggested_command[0].as_str());

	let mut old_entries = Vec::new();
	for command in &split_suggested_command {
		if command.is_empty() {
			continue;
		}
		for old in split_last_command.clone() {
			if command == &old {
				old_entries.push(command.clone());
				break;
			}
		}
	}

	// let mut highlighted = suggested_command.to_string();
	'next: for entry in split_suggested_command.iter_mut() {
		for old in &old_entries {
			if old == entry {
				*entry = entry.cyan().to_string();
				continue 'next;
			}
		}
		*entry = entry.red().bold().to_string();
	}

	let highlighted;
	if privileged
		&& (suggested_command.contains("&&")
			|| suggested_command.contains("||")
			|| suggested_command.contains('>'))
	{
		split_suggested_command[1] =
			format!("{} -c \"", shell).red().bold().to_string() + &split_suggested_command[1];
		let len = split_suggested_command.len() - 1;
		split_suggested_command[len] =
			split_suggested_command[len].clone() + "\"".red().bold().to_string().as_str();
		highlighted = split_suggested_command.join(" ");
	} else {
		highlighted = split_suggested_command.join(" ");
	}

	highlighted
}
