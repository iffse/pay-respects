use crate::shell::Data;
use crate::style::highlight_difference;
use crate::suggestions::{best_match_path, suggest_command};
use crate::system;
use crate::{shell, suggestions};
use colored::Colorize;
use inquire::*;

pub fn suggestion(data: &mut Data) {
	let shell = data.shell.clone();
	let last_command = data.command.clone();

	loop {
		let suggestion = {
			let command = suggest_command(&data);
			if command.is_none() {
				break;
			};

			let mut command = command.unwrap();
			shell::shell_syntax(&shell, &mut command);
			command
		};
		data.update_suggest(&suggestion);
		data.expand_suggest();

		let highlighted_suggestion = {
			let difference = highlight_difference(&shell, &suggestion, &last_command);
			if difference.is_none() {
				break;
			};
			difference.unwrap()
		};

		let execution =
			suggestions::confirm_suggestion(&data, &highlighted_suggestion);
		if execution.is_ok() {
			return;
		} else {
			data.update_command(&suggestion);
			let msg = Some(execution.err().unwrap().split_whitespace().collect::<Vec<&str>>().join(" "));
			data.update_error(msg);

			let retry_message = format!("{}...", t!("retry"));

			eprintln!("\n{}\n", retry_message.cyan().bold());
		}
	}
	eprintln!("{}: {}\n", t!("no-suggestion"), last_command.red());
	eprintln!(
		"{}\n{}",
		t!("contribute"),
		option_env!("CARGO_PKG_REPOSITORY").unwrap_or("https://github.com/iffse/pay-respects/")
	);
}

pub fn cnf(data: &mut Data) {
	let shell = data.shell.clone();
	let last_command = data.command.clone();
	let mut split_command = data.split.clone();

	let executable = split_command[0].as_str();

	let best_match = best_match_path(executable);
	if best_match.is_some() {
		let best_match = best_match.unwrap();
		split_command[0] = best_match;
		let suggestion = split_command.join(" ");
		data.update_suggest(&suggestion);
		data.expand_suggest();

		let highlighted_suggestion =
			highlight_difference(&shell, &suggestion, &last_command).unwrap();
		let _ = suggestions::confirm_suggestion(&data, &highlighted_suggestion);
	} else {
		let package_manager = match system::get_package_manager(&shell) {
			Some(package_manager) => package_manager,
			None => {
				eprintln!("no package manager found");
				return;
			}
		};

		let packages = match system::get_packages(&shell, &package_manager, executable) {
			Some(packages) => packages,
			None => {
				eprintln!("no package found");
				return;
			}
		};

		let ans = Select::new("Select a package to install", packages).prompt();
		let package = match ans {
			Ok(package) => package,
			Err(_) => {
				eprintln!("no package selected");
				return;
			}
		};

		// retry after installing package
		if system::install_package(&shell, &package_manager, &package) {
			let _ = suggestions::confirm_suggestion(&data, &last_command);
		}
	}
}
