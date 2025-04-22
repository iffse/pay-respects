use std::path::Path;
use colored::Colorize;
use inquire::*;
use ui::Color;

use pay_respects_utils::evals::best_matches_path;
use pay_respects_utils::files::best_match_file;

use crate::shell::{shell_evaluated_commands, Data};
use crate::style::highlight_difference;
use crate::suggestions;
use crate::suggestions::suggest_candidates;
use crate::system;

pub fn suggestion(data: &mut Data) {
	let mut last_command;

	loop {
		last_command = data.command.clone();
		suggest_candidates(data);
		if data.candidates.is_empty() {
			break;
		};

		suggestions::select_candidate(data);

		let execution = suggestions::confirm_suggestion(data);
		if execution.is_ok() {
			return;
		} else {
			data.update_command(&data.suggest.clone().unwrap());
			let msg = Some(execution.err().unwrap());
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

pub fn echo(data: &mut Data) {
	suggest_candidates(data);
	if data.candidates.is_empty() {
		return;
	};
	println!("{}", data.candidates.join("<PR_BR>\n"));
}

pub fn noconfirm(data: &mut Data) {
	let mut last_command;

	loop {
		last_command = data.command.clone();
		suggest_candidates(data);
		if data.candidates.is_empty() {
			break;
		};

		let candidate = data.candidates[0].clone();
		eprintln!("{}", highlight_difference(data, &candidate).unwrap());
		data.update_suggest(&candidate);
		data.candidates.clear();

		let execution = suggestions::confirm_suggestion(data);
		if execution.is_ok() {
			return;
		} else {
			data.update_command(&data.suggest.clone().unwrap());
			let msg = Some(execution.err().unwrap());
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

	let best_matches = {
		if executable.contains(std::path::MAIN_SEPARATOR) {
			let file = best_match_file(executable);
			if file.is_some() {
				Some(vec![file.unwrap()])
			} else {
				None
			}
		} else {
			best_matches_path(executable, &data.executables)
		}
	};
	if let Some(best_matches) = best_matches {
		for best_match in best_matches {
			split_command[0] = best_match;
			let suggest = split_command.join(" ");
			data.candidates.push(suggest);
		}
		suggestions::select_candidate(data);

		let status = suggestions::confirm_suggestion(data);
		if status.is_err() {
			let suggest = data.suggest.clone().unwrap();
			data.update_command(&suggest);
			let msg = Some(status.err().unwrap());
			data.update_error(msg);
			let retry_message = format!("{}...", t!("retry"));
			eprintln!("\n{}\n", retry_message.cyan().bold());
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
				eprintln!("{} {}", "pay-respects:".red(), t!("package-not-found"));
				return;
			}
		};

		#[cfg(debug_assertions)]
		eprintln!("packages: {:?}", packages);

		let style = ui::Styled::default();
		let render_config = ui::RenderConfig::default()
			.with_prompt_prefix(style)
			.with_highlighted_option_prefix(ui::Styled::new(">").with_fg(Color::LightBlue))
			.with_scroll_up_prefix(ui::Styled::new("^"))
			.with_scroll_down_prefix(ui::Styled::new("v"))
			.with_option_index_prefix(ui::IndexPrefix::SpacePadded);

		let msg = format!("{}:", t!("install-package")).bold().blue();
		let confirm = format!("[{}]", t!("confirm-yes")).green();
		let hint = format!("{} {} {}", "[↑/↓/j/k]".blue(), confirm, "[ESC]".red());
		eprintln!("{}", msg);
		eprintln!("{}", hint);
		let package = Select::new("\n", packages)
			.with_vim_mode(true)
			.without_help_message()
			.with_render_config(render_config)
			.without_filtering()
			.prompt()
			.unwrap_or_else(|_| std::process::exit(1));

		// retry after installing package
		if system::install_package(data, &package_manager, &package) {
			let status = suggestions::run_suggestion(data, &data.command);
			if status.success() {
				shell_evaluated_commands(&shell, &data.command, true);
			} else {
				shell_evaluated_commands(&shell, &data.command, false);
				data.update_error(None);
				suggestion(data);
			}
		}
	}
}
