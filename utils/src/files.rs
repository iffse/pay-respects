// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::evals::find_similar;
use crate::shell::*;
use crate::strings::replace_escaped_character;
use itertools::Itertools;

pub fn get_path_files() -> Vec<String> {
	let env = std::env::var("_PR_EXECUTABLES");
	if let Ok(env) = env
		&& !env.is_empty()
	{
		return env.split(' ').map(|s| s.to_owned()).collect();
	}

	let path_env = path_env();

	#[cfg(debug_assertions)]
	eprintln!("path_env: {path_env}");

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
			{
				if file_name.ends_with(".dll") {
					continue;
				}
				strip_extension(&mut file_name);
			}

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
		.iter()
		.unique()
		.cloned()
		.collect::<Vec<String>>()
		.into_iter()
		.map(|s| shell_path_post_processing(&s))
		.collect()
}

pub fn best_match_file(input: &str) -> Option<String> {
	let quoted = input.starts_with('\'') || input.starts_with('"') || input.starts_with('`');
	let mut input = if quoted {
		input
			.trim_matches(|c| c == '\'' || c == '"' || c == '`')
			.to_owned()
	} else {
		replace_escaped_character(input, ' ', " ")
	};
	#[cfg(debug_assertions)]
	eprintln!("best_match_file input: {input}");
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
			.map(|file| file.unwrap().file_name().into_string().unwrap())
			.collect::<Vec<String>>();

		#[cfg(debug_assertions)]
		eprintln!("dir_files: {dir_files:?}");

		let best_match = find_similar(&exit_dir, &dir_files);
		best_match.as_ref()?;

		input = format!("{}/{}", input, best_match.unwrap());
		files = match std::fs::read_dir(&input) {
			Ok(files) => files,
			Err(_) => break,
		};
	}

	#[cfg(debug_assertions)]
	{
		eprintln!("best_match_file final input: {input}");
		eprintln!("shell type is: {:?}", get_shell_type());
	}
	let input = shell_path_post_processing(&input);
	#[cfg(debug_assertions)]
	eprintln!("best_match_file final input after shell postprocessing: {input}");
	Some(input)
}

pub fn config_files() -> Vec<String> {
	let mut paths = system_config_path();
	paths.push(user_config_path());

	let mut files = Vec::new();
	for path in paths {
		if std::path::Path::new(&path).exists() {
			files.push(path);
		}
	}
	files
}

fn system_config_path() -> Vec<String> {
	#[cfg(windows)]
	let xdg_config_dirs = std::env::var("PROGRAMDATA")
		.unwrap_or_else(|_| "C:\\ProgramData".to_string())
		.split(';')
		.map(|s| format!("{}/pay-respects/config.toml", s))
		.collect::<Vec<String>>();
	#[cfg(not(windows))]
	let xdg_config_dirs = std::env::var("XDG_CONFIG_DIRS")
		.unwrap_or_else(|_| "/etc/xdg".to_string())
		.split(':')
		.map(|s| format!("{}/pay-respects/config.toml", s))
		.collect::<Vec<String>>();

	xdg_config_dirs
}

fn user_config_path() -> String {
	#[cfg(windows)]
	let xdg_config_home = std::env::var("APPDATA").unwrap();
	#[cfg(not(windows))]
	let xdg_config_home = std::env::var("XDG_CONFIG_HOME")
		.unwrap_or_else(|_| std::env::var("HOME").unwrap() + "/.config");

	format!("{}/pay-respects/config.toml", xdg_config_home)
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
pub fn path_env_sep() -> &'static str {
	if is_msystem() { ":" } else { ";" }
}

#[cfg(windows)]
pub fn path_convert(path: &str) -> String {
	if is_msystem() {
		msys2_conv_path(path).expect("Failed to convert path for msys")
	} else {
		path.to_owned()
	}
}

#[cfg(windows)]
fn strip_extension(file_name: &mut String) {
	let suffixies = [".exe", ".sh", ".ps1"];
	for suffix in suffixies {
		if let Some(file_name_strip) = file_name.strip_suffix(suffix) {
			*file_name = file_name_strip.to_owned();
			break;
		}
	}
}

#[cfg(not(windows))]
fn path_env() -> String {
	std::env::var("PATH").unwrap()
}

#[cfg(not(windows))]
pub fn path_env_sep() -> &'static str {
	":"
}
