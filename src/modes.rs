use crate::shell::{command_output, get_shell, PRIVILEGE_LIST};
use crate::style::highlight_difference;
use crate::suggestions::{best_match_path, split_command};
use crate::system;
use crate::{shell, suggestions};
use colored::Colorize;
use inquire::*;

pub fn suggestion() {
	let shell = get_shell();
	let mut last_command = shell::last_command(&shell).trim().to_string();
	last_command = shell::expand_alias(&shell, &last_command);
	let mut error_msg = {
		let error_msg = std::env::var("_PR_ERROR_MSG");
		if let Ok(error_msg) = error_msg {
			error_msg
		} else {
			command_output(&shell, &last_command)
		}
	};

	error_msg = error_msg
		.split_whitespace()
		.collect::<Vec<&str>>()
		.join(" ");

	loop {
		let suggestion = {
			let command = suggestions::suggest_command(&shell, &last_command, &error_msg);
			if command.is_none() {
				break;
			};

			let mut command = command.unwrap();
			shell::shell_syntax(&shell, &mut command);
			command
		};

		let highlighted_suggestion = {
			let difference = highlight_difference(&shell, &suggestion, &last_command);
			if difference.is_none() {
				break;
			};
			difference.unwrap()
		};

		let execution =
			suggestions::confirm_suggestion(&shell, &suggestion, &highlighted_suggestion);
		if execution.is_ok() {
			return;
		} else {
			last_command = suggestion;
			error_msg = execution.err().unwrap();
			error_msg = error_msg
				.split_whitespace()
				.collect::<Vec<&str>>()
				.join(" ");

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

pub fn cnf() {
	let shell = get_shell();
	let mut last_command = shell::last_command(&shell).trim().to_string();
	last_command = shell::expand_alias(&shell, &last_command);

	let mut split_command = split_command(&last_command);
	let executable = match PRIVILEGE_LIST.contains(&split_command[0].as_str()) {
		true => split_command.get(1).expect(&t!("no-command")).as_str(),
		false => split_command.first().expect(&t!("no-command")).as_str(),
	};

	let best_match = best_match_path(executable);
	if best_match.is_some() {
		let best_match = best_match.unwrap();
		match PRIVILEGE_LIST.contains(&split_command[0].as_str()) {
			true => {
				split_command[1] = best_match;
			}
			false => {
				split_command[0] = best_match;
			}
		}
		let suggestion = split_command.join(" ");

		let highlighted_suggestion =
			highlight_difference(&shell, &suggestion, &last_command).unwrap();
		let _ = suggestions::confirm_suggestion(&shell, &suggestion, &highlighted_suggestion);
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
			let _ = suggestions::confirm_suggestion(&shell, &last_command, &last_command);
		}
	}
}
