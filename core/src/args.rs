use crate::shell::{initialization, Init};
use colored::Colorize;

pub enum Status {
	Continue,
	Exit, // version, help, etc.
	Error,
}

pub fn handle_args() -> Status {
	use Status::*;
	let args = std::env::args().collect::<Vec<String>>();
	if args.len() <= 1 {
		return Continue;
	}
	let mut init = Init::new();
	let mut index = 1;
	while index < args.len() {
		match args[index].as_str() {
			"-h" | "--help" => {
				print_help();
				return Exit;
			}
			"-v" | "--version" => {
				print_version();
				return Exit;
			}
			"-a" | "--alias" => {
				if args.len() > index + 1 {
					if args[index + 1].starts_with('-') {
						init.alias = String::from("f");
					} else {
						init.alias = args[index + 1].clone();
						index += 1;
					}
				} else {
					init.alias = String::from("f");
				}
				init.auto_alias = true;
				index += 1;
			}
			"--nocnf" => {
				init.cnf = false;
				index += 1
			}
			_ => {
				init.shell = args[index].clone();
				index += 1
			}
		}
	}

	if init.shell.is_empty() {
		eprintln!("{}", t!("no-shell"));
		return Error;
	}

	init.binary_path = args[0].clone();

	initialization(&mut init);
	Exit
}

fn print_help() {
	println!(
		"{}",
		t!(
			"help",
			usage = "pay-respects <shell> [--alias [<alias>]] [--nocnf]",
			eval = "Bash / Zsh / Fish".bold(),
			eval_examples = r#"
eval "$(pay-respects bash --alias)"
eval "$(pay-respects zsh --alias)"
pay-respects fish --alias | source
"#,
			manual = "Nushell / PowerShell".bold(),
			manual_examples = r#"
pay-respects nushell
pay-respects pwsh --alias
"#
		)
	);
}

fn print_version() {
	println!(
		"version: {}",
		option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
	);
}
