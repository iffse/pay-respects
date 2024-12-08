mod replaces;
mod rules;

fn main() -> Result<(), std::io::Error>{
	let executable = std::env::var("_PR_COMMAND").unwrap();
	let shell = std::env::var("_PR_SHELL").unwrap();
	let last_command = std::env::var("_PR_LAST_COMMAND").unwrap();
	let error_msg = std::env::var("_PR_ERROR_MSG").unwrap();
	let executables: Vec<String> = {
		let executables = std::env::var("_PR_EXECUTABLES").unwrap();
		executables.split(",").map(|s| s.to_string()).collect()
	};

	rules::runtime_match(&executable, &shell, &last_command, &error_msg, &executables);
	Ok(())
}
