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
