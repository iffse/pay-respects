use crate::shell::command_output;

use pay_respects_utils::{
	evals::{best_matches_path, compare_string, fuzzy_best_n_substring, segment, segment_1},
	files::best_match_file,
	lists::commond_arguments,
	settings::get_trigram_minimum_score,
	shell::shell_path_post_processing,
};

pub enum Functions {
	DesperateFuzzyRecovery,
	DesperateFileLookUp,
	ZoxideIntegration,
}

use Functions::*;

#[allow(unused_variables)]
#[allow(clippy::too_many_arguments)]
pub fn rules_function(
	function: Functions,
	error_msg: &str,
	error_lower: &str,
	shell: &str,
	last_command: &str,
	executables: &[String],
	split: &[String],
	candidates: &mut Vec<String>,
) {
	match function {
		DesperateFuzzyRecovery => desperate_fuzzy_recovery(executables, split, candidates),
		ZoxideIntegration => zoxide_integration(shell, executables, split, candidates),
		DesperateFileLookUp => desperate_file_look_up(split, candidates),
	}
}

pub fn desperate_fuzzy_recovery(
	executables: &[String],
	split: &[String],
	candidates: &mut Vec<String>,
) {
	let mut segments: Vec<String> = Vec::new();
	let mut command: Vec<String> = Vec::new();

	let dict = commond_arguments()
		.iter()
		.map(|s| s.to_string())
		.collect::<Vec<String>>();

	for split in split[1..].iter() {
		let seg = segment(split, &dict);
		for s in seg {
			segments.push(s.to_string());
		}
	}

	#[cfg(debug_assertions)]
	eprintln!(
		"split and segments:\n - split: {:?}\n - segments: {:?}",
		split[1..].iter().collect::<Vec<&String>>(),
		segments
	);

	if executables.contains(&split[0]) || split[0].contains(std::path::MAIN_SEPARATOR) {
		command.push(split[0].to_string());
	} else {
		// we have a problem with the command itself
		if split[0].len() < 3 {
			return;
		}
		if let Some(best_matches) = best_matches_path(&split[0], executables) {
			for best_match in best_matches {
				command.push(best_match);
			}
		} else {
			// introduces some false possitive
			// gitpush -> git pwsh because pwsh is in executables
			let command_segments = segment_1(&split[0], executables);

			if !command_segments.is_empty() {
				for segments in &command_segments {
					command.push(segments.join(" "))
				}
			} else {
				return;
			}
		}
	}

	for command in command.iter() {
		let suggestion = format!("{} {}", command, segments.join(" "));
		candidates.push(suggestion);
	}
}

fn desperate_file_look_up(split: &[String], candidates: &mut Vec<String>) {
	let hints = split[1..]
		.iter()
		.map(|s| {
			if let Some(file) = best_match_file(s) {
				file
			} else {
				s.to_string()
			}
		})
		.collect::<Vec<String>>();

	let joined_hints = hints.join(" ");
	let suggestion = format!("{} {}", split[0], joined_hints);

	candidates.push(suggestion);
}

fn zoxide_integration(
	shell: &str,
	executables: &[String],
	split: &[String],
	candidates: &mut Vec<String>,
) {
	if !executables.contains(&"zoxide".to_string()) {
		return;
	}
	let query_command = "zoxide query -l";
	let hints = split[1..]
		.iter()
		.map(|s| s.to_lowercase())
		.collect::<Vec<String>>();

	let zoxide_output = command_output(shell, query_command);
	let directories = zoxide_output.lines();

	if directories.clone().count() == 0 {
		return;
	}

	let mut filtered_directories = Vec::new();
	for directory in directories.clone() {
		let mut should_add = true;
		for hint in &hints {
			if !directory.to_lowercase().contains(hint) {
				should_add = false;
				break;
			}
		}
		if should_add {
			filtered_directories.push(directory);
		}
	}

	let joined_hints = hints.join(" ");
	if !filtered_directories.is_empty() {
		let mut min_distance = usize::MAX;
		let mut min_idx = usize::MAX;
		// wanted to priotize current directory, but doesn't seems to work well
		// let joined_hints = format!("{}/{}", std::env::current_dir().unwrap().to_str().unwrap(), hints.join(" "));
		for (idx, directory) in filtered_directories.iter().enumerate() {
			let distance = compare_string(&joined_hints, directory);
			if distance < min_distance {
				min_distance = distance;
				min_idx = idx;
			}
		}
		let directory = shell_path_post_processing(filtered_directories[min_idx]);
		candidates.push(format!("cd {}", directory));
	} else {
		let match_candidates = directories.map(|s| s.to_string()).collect::<Vec<String>>();
		let directories = fuzzy_best_n_substring(
			&joined_hints,
			&match_candidates,
			get_trigram_minimum_score(),
			3,
		);
		if let Some(directories) = directories {
			for directory in directories {
				let directory = shell_path_post_processing(&directory);
				candidates.push(format!("cd {}", directory.clone()));
			}
		}
	}
}
