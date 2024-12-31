use crate::shell::Data;
use crate::shell::PRIVILEGE_LIST;
use colored::*;
use pay_respects_utils::evals::split_command;

// to_string() is necessary here, otherwise there won't be color in the output
#[warn(clippy::unnecessary_to_owned)]
pub fn highlight_difference(data: &Data, suggested_command: &str) -> Option<String> {
	// let replaced_newline = suggested_command.replace('\n', r" {{newline}} ");
	let shell = &data.shell;
	let last_command = &data.command;
	let mut split_suggested_command = split_command(suggested_command);
	let split_last_command = split_command(last_command);

	if split_suggested_command == split_last_command {
		return None;
	}
	if split_suggested_command.is_empty() {
		return None;
	}

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
		if entry == "\n" {
			continue;
		}
		for old in &old_entries {
			if old == entry {
				*entry = entry.blue().to_string();
				continue 'next;
			}
		}
		*entry = entry.red().bold().to_string();
	}

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
	}

	if let Some(sudo) = data.privilege.clone() {
		if suggested_command.contains("&&")
			|| suggested_command.contains("||")
			|| suggested_command.contains('>')
		{
			split_suggested_command[0] =
				format!("{} -c \"", shell).blue().to_string() + &split_suggested_command[0];
			let len = split_suggested_command.len() - 1;
			split_suggested_command[len] =
				split_suggested_command[len].clone() + "\"".blue().to_string().as_str();
		}
		split_suggested_command.insert(0, sudo.blue().to_string());
	}

	let highlighted = split_suggested_command.join(" ");

	Some(highlighted.replace(" \n ", "\n"))
}
