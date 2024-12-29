// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::evals::find_similar;

pub fn get_path_files() -> Vec<String> {
	let path_env = path_env();

	if cfg!(debug_assertions) {
		eprintln!("path_env: {path_env}");
	}

	let path_env_sep = path_env_sep();

	let path = path_env.split(path_env_sep).collect::<Vec<&str>>();
	let mut all_executable = vec![];
	for p in path {
		#[cfg(windows)]
		let p = path_convert(p);

		#[cfg(debug_assertions)]
		eprintln!("p={p}");

		let files = match std::fs::read_dir(p) {
			Ok(files) => files,
			Err(_) => continue,
		};
		for file in files {
			let file = file.unwrap();
			#[allow(unused_mut)]
			let mut file_name = file.file_name().into_string().unwrap();

			#[cfg(windows)]
			strip_extension(&mut file_name);

			all_executable.push(file_name);
		}
	}

	#[cfg(debug_assertions)]
	{
		let mut all_executable = all_executable.clone();
		all_executable.sort_unstable();
		eprintln!("all_executable={all_executable:?}");
	}

	all_executable
}

pub fn best_match_file(input: &str) -> Option<String> {
	let mut input = input.trim_matches(|c| c == '\'' || c == '"').to_owned();
	if cfg!(debug_assertions) {
		eprintln!("best_match_file input: {input}");
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

#[cfg(windows)]
fn path_env() -> String {
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

#[cfg(windows)]
fn path_env_sep() -> &'static str {
	if is_msystem() {
		":"
	} else {
		";"
	}
}

#[cfg(windows)]
fn path_convert(path: &str) -> String {
	if is_msystem() {
		msys2_conv_path(path).expect("Failed to convert path for msys")
	} else {
		path.to_owned()
	}
}

#[cfg(windows)]
fn strip_extension(file_name: &str) -> String {
	let mut file_name = file_name.to_owned();
	let suffixies = [".exe", ".sh", ".ps1"];
	for suffix in suffixies {
		if let Some(file_name_strip) = file_name.strip_suffix(suffix) {
			file_name = file_name_strip.to_owned();
			break;
		}
	}

	if !file_name.contains(".") {
		file_name = file_name.to_owned();
	}

	file_name
}

#[cfg(not(windows))]
fn path_env() -> String {
	std::env::var("PATH").unwrap()
}

#[cfg(not(windows))]
fn path_env_sep() -> &'static str {
	":"
}
