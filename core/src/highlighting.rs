use crate::data::Data;
use crate::shell::is_privileged;
use colored::*;
use pay_respects_utils::evals::split_command;
use pay_respects_utils::ui::Colors;

// to_string() is necessary here, otherwise there won't be color in the output
#[warn(clippy::unnecessary_to_owned)]
pub fn highlight_difference(data: &Data, suggested_command: &str, active: bool) -> Option<String> {
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

	let privileged = is_privileged(data, &split_suggested_command[0]);
	let colors = data.config.ui.colors;

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

	let mut skip = false;

	'next: for entry in split_suggested_command.iter_mut() {
		if entry == "#" {
			skip = true;
			continue;
		}
		if entry == "\n" {
			skip = false;
			continue;
		}
		if skip {
			*entry = colors.base(entry).to_string();
			continue;
		}
		for old in &old_entries {
			if old == entry {
				*entry = color_same(entry, active, &colors);
				continue 'next;
			}
		}
		*entry = color_diff(entry, active, &colors);
	}

	if privileged
		&& (suggested_command.contains("&&")
			|| suggested_command.contains("||")
			|| suggested_command.contains('>'))
	{
		split_suggested_command[1] = color_diff(&format!("{} -c \"\n", shell), active, &colors)
			+ split_suggested_command[1].as_str();
		let len = split_suggested_command.len() - 1;
		split_suggested_command[len] =
			split_suggested_command[len].clone() + color_diff("\n\"", active, &colors).as_str();
	}

	if let Some(sudo) = data.privilege.clone() {
		if suggested_command.contains("&&")
			|| suggested_command.contains("||")
			|| suggested_command.contains('>')
		{
			split_suggested_command[0] = color_same(&format!("{} -c \"\n", shell), active, &colors)
				+ split_suggested_command[0].as_str();
			let len = split_suggested_command.len() - 1;
			split_suggested_command[len] =
				split_suggested_command[len].clone() + color_same("\n\"", active, &colors).as_str();
		}
		split_suggested_command.insert(0, color_same(&sudo, active, &colors));
	}

	let highlighted = split_suggested_command.join(" ").replace(" \n ", "\n");

	Some(highlighted)
}

fn color_same(str: &str, active: bool, colors: &Colors) -> String {
	if active {
		str.color(colors.accent).to_string()
	} else {
		colors.base(str).to_string()
	}
}

fn color_diff(str: &str, active: bool, colors: &Colors) -> String {
	if active {
		str.color(colors.wrong).bold().to_string()
	} else {
		colors.base(str).bold().to_string()
	}
}
