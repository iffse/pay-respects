use crate::shell::print_command_with_env;

pub fn handle_args() {
	let args = std::env::args().collect::<Vec<String>>();
	if args.len() > 1 {
		let shell = &args[1];
		let binary_path = &args[0];

		print_command_with_env(shell, binary_path);
	}
}
