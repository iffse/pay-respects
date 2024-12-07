use crate::suggestions::find_similar;

pub fn get_path_files() -> Vec<String> {
	let path_env = {
		#[cfg(windows)]
		{
			if is_msystem() {
				String::from_utf8_lossy(
					&std::process::Command::new("bash")
						.arg("-c")
						.arg("echo $PATH")
						.output()
						.unwrap()
						.stdout,
				)
				.trim()
				.to_owned()
			} else {
				std::env::var("PATH").unwrap()
			}
		}
		#[cfg(not(windows))]
		{
			std::env::var("PATH").unwrap()
		}
	};

	if cfg!(debug_assertions) {
		eprintln!("path_env: {path_env}");
	}

	let path_env_sep = {
		#[cfg(windows)]
		if is_msystem() {
			":"
		} else {
			";"
		}
		#[cfg(not(windows))]
		{
			":"
		}
	};

	let path = path_env.split(path_env_sep).collect::<Vec<&str>>();
	let mut all_executable = vec![];
	for p in path {
		#[cfg(windows)]
		let p = if is_msystem() {
			msys2_conv_path(p).expect("Failed to convert path for msys")
		} else {
			p.to_owned()
		};

		if cfg!(debug_assertions) {
			eprintln!("p={p}");
		}

		let files = match std::fs::read_dir(p) {
			Ok(files) => files,
			Err(_) => continue,
		};
		for file in files {
			let file = file.unwrap();
			#[allow(unused_mut)]
			let mut file_name = file.file_name().into_string().unwrap();

			#[cfg(windows)]
			{
				let mut ok = false;
				let suffixies = [".exe", ".sh", ".ps1"];
				for suffix in suffixies {
					if let Some(file_name_strip) = file_name.strip_suffix(suffix) {
						file_name = file_name_strip.to_owned();
						ok = true;
						break;
					}
				}

				if !file_name.contains(".") {
					ok = true;
				}

				if !ok {
					continue;
				}
			}

			all_executable.push(file_name);
		}
	}

	if cfg!(debug_assertions) {
		let mut all_executable = all_executable.clone();
		all_executable.sort_unstable();
		eprintln!("all_executable={all_executable:?}");
	}

	all_executable
}

pub fn get_best_match_file(input: &str) -> Option<String> {
	let mut input = input.trim_matches(|c| c == '\'' || c == '"').to_owned();
	if cfg!(debug_assertions) {
		eprintln!("get_best_match_file input: {input}");
	}
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

		let best_match = find_similar(&exit_dir, &dir_files, Some(2));
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
		.map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
#[cfg(windows)]
fn is_msystem() -> bool {
	std::env::var("MSYSTEM").is_ok()
}
