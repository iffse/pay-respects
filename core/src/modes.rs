use crate::shell::Data;
use crate::suggestions::suggest_candidates;
use crate::system;
use crate::{shell, suggestions};
use colored::Colorize;
use inquire::*;
use pay_respects_utils::evals::best_match_path;

use std::path::Path;

pub fn suggestion(data: &mut Data) {
	let shell = data.shell.clone();
	let mut last_command;

	loop {
		last_command = data.command.clone();
		suggest_candidates(data);
		if data.candidates.is_empty() {
			break;
		};

		for candidate in &mut data.candidates {
			shell::shell_syntax(&shell, candidate);
		}

		suggestions::select_candidate(data);

		let execution = suggestions::confirm_suggestion(data);
		if execution.is_ok() {
			return;
		} else {
			data.update_command(&data.suggest.clone().unwrap());
			let msg = Some(
				execution
					.err()
					.unwrap()
					.split_whitespace()
					.collect::<Vec<&str>>()
					.join(" "),
			);
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
	let mut split_command = data.split.clone();

	let executable = split_command[0].as_str();
	let shell_msg = format!("{}:", shell);
	eprintln!(
		"{} {}: {}\n",
		shell_msg.bold().red(),
		t!("command-not-found"),
		executable
	);

	let best_match = best_match_path(executable, &data.executables);
	if best_match.is_some() {
		let best_match = best_match.unwrap();
		split_command[0] = best_match;
		let suggest = split_command.join(" ");

		data.candidates.push(suggest.clone());
		suggestions::select_candidate(data);

		let status = suggestions::confirm_suggestion(data);
		if status.is_err() {
			data.update_command(&suggest);
			let msg = Some(
				status
					.err()
					.unwrap()
					.split_whitespace()
					.collect::<Vec<&str>>()
					.join(" "),
			);
			data.update_error(msg);
			suggestion(data);
		}
	} else {
		let package_manager = match system::get_package_manager(data) {
			Some(package_manager) => match package_manager.as_str() {
				"apt" => {
					let cnf_dirs = [
						"/usr/lib/",
						"/data/data/com.termux/files/usr/libexec/termux/",
					];
					let mut package_manager = package_manager;
					for bin_dir in &cnf_dirs {
						let bin = format!("{}{}", bin_dir, "command-not-found");
						if Path::new(&bin).exists() {
							package_manager = bin;
							break;
						}
					}
					package_manager
				}
				_ => package_manager,
			},
			None => {
				return;
			}
		};

		#[cfg(debug_assertions)]
		eprintln!("package_manager: {}", package_manager);

		let packages = match system::get_packages(data, &package_manager, executable) {
			Some(packages) => packages,
			None => {
				eprintln!("{}: {}", "pay-respects".red(), t!("package-not-found"));
				return;
			}
		};

		#[cfg(debug_assertions)]
		eprintln!("packages: {:?}", packages);

		let style = ui::Styled::default();
		let render_config = ui::RenderConfig::default().with_prompt_prefix(style);
		let msg = format!("{}:", t!("install-package")).bold().blue();
		let confirm = format!("[{}]", t!("confirm-yes")).green();
		let hint = format!("{} {} {}", "[↑/↓]".blue(), confirm, "[Ctrl+C]".red());
		eprintln!("{}", msg);
		eprintln!("{}", hint);
		let package = Select::new("\n", packages)
			.with_vim_mode(true)
			.without_help_message()
			.with_render_config(render_config)
			.without_filtering()
			.prompt()
			.unwrap();

		// retry after installing package
		if system::install_package(data, &package_manager, &package) {
			let status = suggestions::run_suggestion(data, &data.command);
			if !status.success() {
				data.update_error(None);
				suggestion(data);
		}
	}
}
