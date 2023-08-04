use crate::shell::initialization;

pub fn handle_args() {
	let args = std::env::args().collect::<Vec<String>>();
	if args.len() <= 1 {
		return;
	}
	let mut auto_aliasing = String::new();
	let mut shell = String::new();
	let mut index = 0;
	while index < args.len() {
		match args[index].as_str() {
			"-h" | "--help" => {
				print_help();
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
			_ => {
				shell = args[index].clone();
				index += 1
			}
		}
	}

	if shell.is_empty() {
		eprintln!("No shell specified. Please specify a shell.");
		std::process::exit(1);
	}

	let binary_path = &args[0];

	initialization(&shell, binary_path, &auto_aliasing);
}

fn print_help() {
	let help_message = String::from(
		"
Usage: pay_respects [your shell] [--alias [alias]]

Example 1, manual aliasing: `pay_respects bash`

The command will output the command that you can use to execute the binary with
the correct environment. You can alias such output to a shorter key. Such as
alias f=$(pay_respects bash)

Example 2, auto aliasing: `pay_respects bash --alias f`

The command will output a declaration that can be directly embedded in your
config file with `eval $(pay_respects bash --alias)`. For fish, use
`pay_respects fish --alias | source` instead.
	",
	);
	println!("{}", help_message);
	std::process::exit(0);
}
