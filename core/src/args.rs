use std::path::Path;

use crate::{init::Init, shell::initialization};
use colored::Colorize;
use pay_respects_utils::shell::{detect_shell_type, normalize_shell_type};
use pay_respects_utils::strings::print_error;

pub enum Status {
	Continue,
	Exit, // version, help, etc.
	Error,
}

pub fn handle_args(args: impl IntoIterator<Item = String>) -> Status {
	let mut iter = args.into_iter().peekable();
	let mut init = Init::new();
	if let Some(binary_path) = iter.next() {
		init.binary_path = binary_path;
	}

	if iter.peek().is_none() {
		return Status::Continue;
	}

	while let Some(arg) = iter.next() {
		match arg.as_str() {
			"-h" | "--help" => {
				print_help();
				return Status::Exit;
			}
			"-v" | "--version" => {
				print_version();
				return Status::Exit;
			}
			"-a" | "--alias" => match iter.peek() {
				Some(next_arg) if !next_arg.starts_with('-') => {
					init.alias = next_arg.to_string();
					iter.next();
				}
				_ => init.alias = String::from("f"),
			},
			"-s" | "--shell-type" => match iter.next() {
				Some(shell_type) if !shell_type.starts_with('-') => {
					init.shell_type = shell_type;
				}
				_ => {
					print_error("--shell-type requires argument.");
					return Status::Error;
				}
			},
			"--nocnf" => init.cnf = false,
			"-P" | "--prompt-prefix" => match iter.peek() {
				Some(next_arg) if !next_arg.starts_with('-') => {
					init.cmd_prefix = Some(next_arg.to_string());
					iter.next();
				}
				_ => {
					print_error("--prompt-prefix requires argument.");
					return Status::Error;
				}
			},
			"--migrate-rules" => {
				let Some(path) = iter.next() else {
					print_error("--migrate-rules requires a file.");
					return Status::Error;
				};
				if iter.peek().is_some() {
					print_error("--migrate-rules accepts one file at a time.");
					return Status::Error;
				}
				match pay_respects_rule_engine::migrate_file(Path::new(&path)) {
					Ok(content) => {
						print!("{}", content);
						return Status::Exit;
					}
					Err(err) => {
						print_error(&format!("Failed to migrate {}: {}", path, err));
						return Status::Error;
					}
				}
			}
			_ => {
				if arg.starts_with('-') {
					print_error(&format!("Unknown option: {}", arg));
					return Status::Error;
				} else {
					init.shell = arg
				}
			}
		}
	}

	if init.shell.is_empty() && init.shell_type.is_empty() {
		print_error(&t!("no-shell"));
		return Status::Error;
	}

	if init.shell_type.is_empty() {
		match detect_shell_type(&init.shell) {
			Some(shell_type) => init.shell_type = shell_type.to_string(),
			None => {
				print_error(&format!(
					"Unknown shell type for '{}'. Use --shell-type <type>.",
					init.shell
				));
				return Status::Error;
			}
		}
	} else {
		let requested_shell = init.shell_type.clone();
		match normalize_shell_type(&init.shell_type) {
			Some(shell_type) => init.shell_type = shell_type.to_string(),
			None => {
				print_error(&format!("Unknown shell type: {}", init.shell_type));
				return Status::Error;
			}
		}
		if init.shell.is_empty() {
			init.shell = requested_shell;
		}
	}

	initialization(&mut init);
	Status::Exit
}

fn print_help() {
	let init = "pay-respects [<shell-binary>] [OPTIONS]";
	let options = r#"Options:
	-h, --help                   Print help information
	-v, --version                Print version information
	-a, --alias [<alias>]        Set alias for the function (default: f)
	-s, --shell-type <type>      Shell type when it differs from the binary name
	    --nocnf                  Do not load cnf file
	-P, --prompt-prefix <prefix> Force a prompt prefix (e.g. ">" or "❯")
	    --migrate-rules <file>   Convert a version 1 rule file to version 2
"#;
	println!(
		"{}",
		t!(
			"help",
			usage = format!("{}\n\n{}", init, options),
			eval = "Bash / Zsh / Fish / Dash / Xonsh / Elvish"
				.bold()
				.to_string(),
			eval_examples = r#"
eval "$(pay-respects bash)"
eval "$(pay-respects zsh)"
pay-respects fish | source
eval "$(pay-respects dash)"
execx($(pay-respects xonsh))
eval (pay-respects elvish | slurp)
"#,
			manual = "Nushell / PowerShell".bold().to_string(),
			manual_examples = r#"
pay-respects nu
pay-respects pwsh
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
	if let Some(lib) = lib {
		println!("Default lib directory: {}", lib);
	}
	let package_manager = option_env!("_DEF_PR_PACKAGE_MANAGER").map(|dir| dir.to_string());
	if let Some(package_manager) = package_manager {
		println!("Default package manager: {}", package_manager);
	}
}

#[cfg(test)]
mod tests {
	use super::{Status, handle_args};

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
			[String::new(), String::from("dash")],
			[String::new(), String::from("xonsh")],
			[String::new(), String::from("elvish")],
		] {
			println!("Arguments {:?} should return Exit", args);
			assert!(matches!(handle_args(args), Status::Exit));
		}

		for args in [
			vec![String::new(), String::from("fish"), String::from("--alias")],
			vec![String::new(), String::from("bash"), String::from("--nocnf")],
			vec![String::new(), String::from("zsh-fork")],
			vec![
				String::new(),
				String::from("zsh-fork"),
				String::from("--shell-type"),
				String::from("zsh"),
			],
			vec![
				String::new(),
				String::from("--shell-type"),
				String::from("zsh"),
			],
		] {
			println!("Arguments {:?} should return Exit", args);
			assert!(matches!(handle_args(args), Status::Exit));
		}

		for args in [
			vec![String::new(), String::from("-a")],
			vec![String::new(), String::from("--alias")],
			vec![String::new(), String::from("--nocnf")],
			vec![String::new(), String::from("--migrate-rules")],
			vec![String::new(), String::from("--shell-type")],
			vec![String::new(), String::from("unknown-shell")],
			vec![
				String::new(),
				String::from("zsh-fork"),
				String::from("--shell-type"),
				String::from("unknown"),
			],
		] {
			println!("Arguments {:?} should return Error", args);
			assert!(matches!(handle_args(args), Status::Error));
		}

		assert!(matches!(
			handle_args([
				String::new(),
				String::from("--migrate-rules"),
				String::from("one.toml"),
				String::from("two.toml"),
			]),
			Status::Error
		));

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
