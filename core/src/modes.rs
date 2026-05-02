use colored::Colorize;
use pay_respects_select::select_simple;
use pay_respects_utils::strings::{format_prefix, print_error, remove_color_codes};
use std::path::Path;
use std::process::exit;

use pay_respects_utils::evals::best_matches;
use pay_respects_utils::files::best_match_file;

use crate::data::Data;
use crate::highlighting::highlight_difference;
use crate::shell::{add_candidates_no_dup, shell_evaluated_commands};
use crate::suggestions::{inline_suggestion, suggest_candidates};
use crate::system;
use crate::{config, suggestions};

pub fn suggestion(data: &mut Data) {
	let mut last_command;

	loop {
		last_command = data.command.clone();
		suggest_candidates(data);
		if data.candidates.is_empty() {
			break;
		};

		suggestions::select_candidate(data);

		let execution = suggestions::execute_suggestion(data);
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

pub fn inline(data: &mut Data) {
	inline_suggestion(data);

	if data.candidates.is_empty() {
		return;
	}
	println!("{} ", data.candidates[0]);
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
		let highlighted = highlight_difference(data, &candidate, true).unwrap();

		let output = if let Some(prefix) = &data.prompt_prefix {
			data.input_command = remove_color_codes(&highlighted);
			format_prefix(prefix, &highlighted)
		} else {
			candidate.clone()
		};
		eprintln!("{}", output);
		data.update_suggest(&candidate);
		data.candidates.clear();

		let execution = suggestions::execute_suggestion(data);
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
	let mut split = data.split.clone();

	let executable = split[0].clone();
	let shell_msg = format!("{}:", shell);
	eprintln!(
		"{} {}: {}\n",
		shell_msg.bold().red(),
		t!("command-not-found"),
		executable
	);

	let mut candidates: Vec<String> = Vec::new();
	// too aggresive
	// desperate_fuzzy_recovery(&data.executables, &split, &mut candidates);

	let best_matches = {
		if executable.contains(std::path::MAIN_SEPARATOR) {
			let file = best_match_file(&executable);
			if let Some(file) = file {
				let currrent_dir_prefix = format!(".{}", std::path::MAIN_SEPARATOR);
				let file = if executable.starts_with(&currrent_dir_prefix) {
					format!(".{}{}", std::path::MAIN_SEPARATOR, file)
				} else {
					file
				};
				Some(vec![file])
			} else {
				None
			}
		} else {
			best_matches(&executable, &data.executables)
		}
	};

	if let Some(best_matches) = best_matches {
		for best_match in best_matches {
			if best_match == executable {
				continue;
			}
			split[0] = best_match;
			let suggest = split.join(" ");
			candidates.push(suggest);
		}
	}

	add_candidates_no_dup(&data.command, &mut data.candidates, &candidates);

	if !data.candidates.is_empty() {
		suggestions::select_candidate(data);

		let status = suggestions::execute_suggestion(data);
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
		data.config.set_package_manager(&package_manager);

		#[cfg(debug_assertions)]
		eprintln!("package_manager: {}", package_manager);

		let packages = match system::get_packages(data, &package_manager, &executable) {
			Some(packages) => packages,
			None => {
				eprintln!("{} {}", "pay-respects:".red(), t!("package-not-found"));
				return;
			}
		};

		#[cfg(debug_assertions)]
		eprintln!("packages: {:?}", packages);

		// change default install method based on package manager
		data.config.set_install_method();

		let packages = packages
			.iter()
			.map(|p| system::install_string(data, &package_manager, p))
			.collect::<Vec<String>>();

		let msg = format!("{}:", t!("install-package")).bold().blue();
		let confirm = format!("[{}]", t!("confirm-yes")).green();
		let hint = format!("{} {} {}", "[↑/↓/j/k]".blue(), confirm, "[ESC]".red());

		let prelude = format!("{}\n\r{}", msg, hint);
		let selection = select_simple(&prelude, &packages).unwrap_or_else(|err| {
			print_error(&format!("Selection failed: {}", err));
			exit(1);
		});

		let package = packages[selection].to_string();

		let install_method = &data.config.install_method;
		if install_method == &config::InstallMethod::Shell {
			// let the shell handle the installation and place the user in a shell
			// environment with the package installed
			println!(
				"{}",
				system::install_package_shell(data, &package_manager, &package)
			);
			return;
		}

		// retry after installing package
		if system::install_package(data, &package_manager, &package) {
			let output = if let Some(prefix) = &data.prompt_prefix {
				format_prefix(prefix, &data.input_command)
			} else {
				data.input_command.clone()
			};
			eprintln!("\n{}", output.cyan());
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
