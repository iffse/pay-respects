use crate::shell::{initialization, Init};
use colored::Colorize;

pub enum Status {
	Continue,
	Exit, // version, help, etc.
	Error,
}

pub fn handle_args(args: impl IntoIterator<Item = String>) -> Status {
	let args: Vec<_> = args.into_iter().collect();
	if args.len() <= 1 {
		return Status::Continue;
	}

	let mut init = Init::new();
	let mut index = 1;
	while index < args.len() {
		match args[index].as_str() {
			"-h" | "--help" => {
				print_help();
				return Status::Exit;
			}
			"-v" | "--version" => {
				print_version();
				return Status::Exit;
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
		return Status::Error;
	}

	init.binary_path = args[0].clone();

	initialization(&mut init);
	Status::Exit
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
pay-respects nushell --alias
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
	let lib = option_env!("_DEF_PR_LIB").map(|dir| dir.to_string());
	if lib.is_some() {
		println!("Default lib directory: {}", lib.unwrap());
	}
}

#[cfg(test)]
mod tests {
	use super::{handle_args, Status};

	#[test]
	fn test_handle_args() {
		assert!(matches!(
			handle_args([String::from("pay-respects")]),
			Status::Continue
		));

		for args in [
			[String::new(), String::from("-h")],
			[String::new(), String::from("--help")],
			[String::new(), String::from("-v")],
			[String::new(), String::from("--version")],
			[String::new(), String::from("zsh")],
		] {
			println!("Arguments {:?} should return Exit", args);
			assert!(matches!(handle_args(args), Status::Exit));
		}

		for args in [
			[String::new(), String::from("fish"), String::from("--alias")],
			[String::new(), String::from("bash"), String::from("--nocnf")],
		] {
			println!("Arguments {:?} should return Exit", args);
			assert!(matches!(handle_args(args), Status::Exit));
		}

		for args in [
			[String::new(), String::from("-a")],
			[String::new(), String::from("--alias")],
			[String::new(), String::from("--nocnf")],
		] {
			println!("Arguments {:?} should return Error", args);
			assert!(matches!(handle_args(args), Status::Error));
		}

		for args in [
			[String::new(), String::from("-a"), String::from("--nocnf")],
			[
				String::new(),
				String::from("--alias"),
				String::from("--nocnf"),
			],
		] {
			println!("Argument {:?} should return Error", args);
			assert!(matches!(handle_args(args), Status::Error));
		}
	}
}
