mod replaces;
mod rules;

fn main() -> Result<(), std::io::Error> {
	let executable = std::env::var("_PR_COMMAND").expect("_PR_COMMAND not set");
	let shell = std::env::var("_PR_SHELL").expect("_PR_SHELL not set");
	let last_command = std::env::var("_PR_LAST_COMMAND").expect("_PR_LAST_COMMAND not set");
	let error_msg = std::env::var("_PR_ERROR_MSG").expect("_PR_ERROR_MSG not set");
	let executables: Vec<String> = {
		let executables = std::env::var("_PR_EXECUTABLES").expect("_PR_EXECUTABLES not set");
		executables.split(" ").map(|s| s.to_string()).collect()
	};

	#[cfg(debug_assertions)]
	{
		eprintln!("shell: {}", shell);
		eprintln!("executable: {}", executable);
		eprintln!("last_command: {}", last_command);
		eprintln!("error_msg: {}", error_msg);
		eprintln!("executables: {:?}", executables);
	}

	rules::runtime_match(&executable, &shell, &last_command, &error_msg, &executables);
	Ok(())
}
