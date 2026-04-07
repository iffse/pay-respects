use crate::{data::{Data, PRIVILEGE_LIST}, integrations::zoxide_integration};

use pay_respects_utils::{
	evals::{best_match, best_matches, segment, segment_1},
	files::best_match_file,
	lists::commond_arguments,
	settings::get_search_threshold,
};

pub enum Functions {
	DesperateFileLookUp,
	DesperateFuzzyRecovery,
	SetPrivilege,
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
	data: &Data,
) {
	match function {
		DesperateFileLookUp => desperate_file_look_up(split, candidates),
		DesperateFuzzyRecovery => desperate_fuzzy_recovery(executables, split, candidates),
		SetPrivilege => set_privilege(data, executables, last_command, candidates),
		ZoxideIntegration => zoxide_integration(shell, executables, split, candidates),
	}
}

fn set_privilege(data: &Data, executables: &[String], last_command: &str, candidates: &mut Vec<String>) {
	if data.privilege.is_some() {
		return;
	}

	if let Some(privilege) = &data.config.privilege {
		candidates.push(format!("{} {}", privilege, last_command));
		return;
	}

	PRIVILEGE_LIST.iter().for_each(|privilege| {
		if executables.contains(&privilege.to_string()) {
			candidates.push(format!("{} {}", privilege, last_command));
		}
	});
}

pub fn desperate_fuzzy_recovery(
	executables: &[String],
	split: &[String],
	candidates: &mut Vec<String>,
) {
	// naive approach for executable only
	if std::env::var("_PR_NO_DESPERATE").is_ok() {
		if executables.contains(&split[0]) || split[0].contains(std::path::MAIN_SEPARATOR) {
			return;
		}
		let best_matches = best_matches(&split[0], executables);
		if let Some(best_matches) = best_matches {
			for best_match in best_matches {
				candidates.push(format!("{} {}", best_match, split[1..].join(" ")));
			}
		}
		return;
	}

	let mut split = split.to_vec();
	let mut segments: Vec<String> = Vec::new();
	let mut command: Vec<String> = Vec::new();

	let dict = commond_arguments()
		.iter()
		.map(|s| s.to_string())
		.collect::<Vec<String>>();

	let mut valid_command =
		executables.contains(&split[0]) || split[0].contains(std::path::MAIN_SEPARATOR);

	let mut head = split[0].clone();
	if !valid_command && head.len() < get_search_threshold() {
		// append head until its length is greater than the threshold
		for i in 2..split.len() + 1 {
			head = split[..i].join("");

			#[cfg(debug_assertions)]
			eprintln!("i: {}, head: {}, lengh: {}", i, head, head.len());

			if head.len() >= get_search_threshold() {
				split = std::iter::once(&head)
					.chain(split[i..].iter())
					.cloned()
					.collect::<Vec<String>>();

				#[cfg(debug_assertions)]
				eprintln!("split: {:?}", split);
				break;
			}
		}
		valid_command = executables.contains(&split[0]);
	}

	for split in split[1..].iter() {
		// don't segment if it's a string (quoted), or a flag that contains `-`, `=`, or `=` after the first two characters (e.g., `--flag=value` or `-f=value`)
		let is_string = split.starts_with('"')
			|| split.starts_with('\'')
			|| split.starts_with('`')
			|| split.contains(std::path::MAIN_SEPARATOR);
		if is_string {
			segments.push(split.to_string());
			continue;
		}

		let prefix_dash_count = split.chars().take_while(|&c| c == '-').count();
		let is_flag = prefix_dash_count > 0 && prefix_dash_count <= 2;
		let is_flag_with_value = is_flag && split[2..].contains("=");

		// if is_flag_with_value {
		// 	segments.push(split.to_string());
		// 	continue;
		// }

		if is_flag_with_value {
			// split into flag and value, find the best match for the flag, and then rejoin them
			let flag_part = &split[..split.find('=').unwrap()];
			let value_part = &split[split.find('=').unwrap() + 1..];
			let best_match = best_match(&flag_part[prefix_dash_count - 1..], &dict);
			if let Some(best_match) = best_match {
				segments.push(format!(
					"{}{}={}",
					&flag_part[..prefix_dash_count],
					best_match,
					value_part
				));
			} else {
				segments.push(split.to_string());
			}
		} else if is_flag {
			// don't segment but find the best match
			let best_match = best_match(&split[prefix_dash_count - 1..], &dict);
			if let Some(best_match) = best_match {
				segments.push(format!("{}{}", &split[..prefix_dash_count], best_match));
			} else {
				segments.push(split.to_string());
			}
		} else {
			let seg = segment(split, &dict);
			for s in seg {
				segments.push(s.to_string());
			}
		}
	}

	#[cfg(debug_assertions)]
	eprintln!(
		"split and segments:\n - split: {:?}\n - segments: {:?}",
		split[0..].iter().collect::<Vec<&String>>(),
		segments
	);

	if valid_command {
		command.push(split[0].to_string());
	} else {
		// we have a problem with the command itself
		if split[0].len() < get_search_threshold() {
			return;
		}
		if let Some(best_matches) = best_matches(&split[0], executables) {
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
	if std::env::var("_PR_NO_DESPERATE").is_ok() {
		return;
	}
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
