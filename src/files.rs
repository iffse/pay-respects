use crate::suggestions::find_similar;

pub fn get_path_files() -> Vec<String> {
	let path_env = if cfg!(windows) {
		String::from_utf8_lossy(
			&std::process::Command::new("/usr/bin/bash")
				.arg("-c")
				.arg("echo $PATH")
				.output()
				.unwrap()
				.stdout,
		)
		.into_owned()
	} else {
		std::env::var("PATH").unwrap()
	};

	if cfg!(debug_assertions) {
		eprintln!("path_env: {path_env}");
	}

	let path = path_env.split(':').collect::<Vec<&str>>();
	let mut all_executable = vec![];
	for p in path {
		#[cfg(windows)]
		let p = msys2_conv_path(p).expect("Failed to convert path for msys");

		let files = match std::fs::read_dir(p) {
			Ok(files) => files,
			Err(_) => continue,
		};
		for file in files {
			let file = file.unwrap();
			let file_name = file.file_name().into_string().unwrap();
			all_executable.push(file_name);
		}
	}
	all_executable
}

pub fn get_best_match_file(input: &str) -> Option<String> {
	let mut input = input.trim_matches(|c| c == '\'' || c == '"').to_owned();
	let mut exit_dirs = Vec::new();
	let mut files = loop {
		match std::fs::read_dir(&input) {
			Ok(files) => break files,
			Err(_) => {
				if let Some((dirs, exit_dir)) = input.rsplit_once(std::path::MAIN_SEPARATOR) {
					exit_dirs.push(exit_dir.to_owned());
					input = dirs.to_owned();
				} else {
					exit_dirs.push(input.to_owned());
					input = ".".to_owned();
					break std::fs::read_dir("./").unwrap();
				}
			}
		}
	};

	while let Some(exit_dir) = exit_dirs.pop() {
		let dir_files = files
			.map(|file| {
				let file = file.unwrap();

				file.file_name().into_string().unwrap().replace(' ', "\\ ")
			})
			.collect::<Vec<String>>();

		let best_match = find_similar(&exit_dir, &dir_files);
		best_match.as_ref()?;

		input = format!("{}/{}", input, best_match.unwrap());
		files = match std::fs::read_dir(&input) {
			Ok(files) => files,
			Err(_) => return Some(input),
		};
	}

	Some(input)
}

#[cfg(windows)]
fn msys2_conv_path(p: &str) -> std::io::Result<String> {
	std::process::Command::new("cygpath")
		.arg("-w")
		.arg(p)
		.output()
		.map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
}
