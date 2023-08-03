pub fn get_path_files() -> Vec<String> {
	let path = std::env::var("PATH").unwrap();
	let path = path.split(':').collect::<Vec<&str>>();
	let mut all_executable = vec![];
	for p in path {
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

pub fn get_directory_files(input: &str) -> Vec<String> {
	let mut input = input.trim_matches(|c| c == '\'' || c == '"').to_owned();
	let files = loop {
		match std::fs::read_dir(&input) {
			Ok(files) => break files,
			Err(_) => {
				if let Some((dirs, _)) = input.rsplit_once('/') {
					input = dirs.to_owned();
				} else {
					break std::fs::read_dir("./").unwrap();
				}
			}
		}
	};

	let mut all_files = vec![];
	for file in files {
		let file = file.unwrap();
		let file_name = file.path().to_str().unwrap().to_owned();
		all_files.push(file_name);
	}
	all_files
}
