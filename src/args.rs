use std::io::prelude::*;

pub fn handle_args() {
	let args = std::env::args().collect::<Vec<String>>();
	if args.len() > 1 {
		let shell = &args[1];
		let binary_path = &args[0];
		let last_command;
		let alias;

		match shell.as_str() {
			"bash" => {
				last_command = "$(history 2)";
				alias = "$(alias)"
			}
			"zsh" => {
				last_command = "$(fc -ln -1)";
				alias = "$(alias)"
			}
			"fish" => {
				last_command = "$(history | head -n 1)";
				alias = "$(alias)";
			}
			"nu" | "nush" | "nushell" => {
				last_command = "(history | last).command";
				alias = "\"\"";
				let command = format!(
					"with-env {{ _PR_LAST_COMMAND : {},\
					_PR_ALIAS : {},\
					_PR_SHELL : nu }} \
					{{ {} }}",
					last_command, alias, binary_path
				);
				println!("{}\n", command);
				println!("Add following to your config file? (Y/n)");
				let alias = format!("alias f = {}", command);
				println!("{}", alias);
				let mut input = String::new();
				std::io::stdin().read_line(&mut input).unwrap();
				match input.trim() {
					"Y" | "y" | "" => {
						let output = std::process::Command::new("nu")
							.arg("-c")
							.arg("echo $nu.config-path")
							.output()
							.expect("Failed to execute process");
						let config_path= String::from_utf8_lossy(&output.stdout);
						let mut file = std::fs::OpenOptions::new()
							.write(true)
							.append(true)
							.open(config_path.trim())
							.expect("Failed to open config file");

						writeln!(file, "{}", alias).expect("Failed to write to config file");
					},
					"n" | _ => std::process::exit(0),
				};
				std::process::exit(0);
			}
			_ => {
				println!("Unknown shell: {}", shell);
				std::process::exit(1);
			}
		}

		println!(
			"\
			_PR_LAST_COMMAND=\"{}\" \
			_PR_ALIAS=\"{}\" \
			_PR_SHELL=\"{}\" \
			\"{}\"",
			last_command, alias, shell, binary_path
		);
		std::process::exit(0);
	}
}
