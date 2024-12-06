use crate::shell::initialization;

pub enum Status {
	Continue,
	Exit, // version, help, etc.
	Error,
}

// returns true if should exit
pub fn handle_args() -> Status {
	use Status::*;
	let args = std::env::args().collect::<Vec<String>>();
	if args.len() <= 1 {
		return Continue;
	}
	let mut auto_aliasing = String::new();
	let mut shell = String::new();
	let mut cnf = true;
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
						auto_aliasing = String::from("f");
					} else {
						auto_aliasing = args[index + 1].clone();
						index += 1;
					}
				} else {
					auto_aliasing = String::from("f");
				}
				index += 1;
			}
			"--noncf" => {
				cnf = false;
				index += 1
			}
			_ => {
				shell = args[index].clone();
				index += 1
			}
		}
	}

	if shell.is_empty() {
		eprintln!("{}", t!("no-shell"));
		return Error;
	}

	let binary_path = &args[0];

	initialization(&shell, binary_path, &auto_aliasing, cnf);
	Exit
}

fn print_help() {
	println!(
		"{}",
		t!(
			"help",
			manual = "pay-respects bash",
			manual_example = "alias f=$(pay-respects bash)",
			auto = "pay-respects bash --alias f",
			auto_example = "eval $(pay-respects bash --alias f)",
			auto_example_fish = "pay-respects fish --alias | source",
		)
	);
}

fn print_version() {
	println!(
		"version: {}",
		option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")
	);
	println!("compile features:");
	#[cfg(feature = "runtime-rules")]
	{
		println!("  - runtime-rules");
	}
	#[cfg(feature = "request-ai")]
	{
		println!("  - request-ai");
	}
	#[cfg(feature = "libcurl")]
	{
		println!("  - libcurl");
	}
}
