use crate::shell::command_output;

use pay_respects_utils::{evals::compare_string, shell::shell_path_post_processing};

pub enum Functions {
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
		ZoxideIntegration => zoxide_integration(shell, executables, split, candidates),
	}
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

	if filtered_directories.is_empty() {
		return;
	}

	let mut min_distance = usize::MAX;
	let mut min_idx = usize::MAX;
	// wanted to priotize current directory, but doesn't seems to work well
	// let joined_hints = format!("{}/{}", std::env::current_dir().unwrap().to_str().unwrap(), hints.join(" "));
	let joined_hints = hints.join(" ");
	for (idx, directory) in filtered_directories.iter().enumerate() {
		let distance = compare_string(&joined_hints, directory);
		if distance < min_distance {
			min_distance = distance;
			min_idx = idx;
		}
	}
	let directory = shell_path_post_processing(filtered_directories[min_idx]);
	candidates.push(format!("cd {}", directory));
}
