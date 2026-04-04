use pay_respects_utils::{
	evals::{compare_string, fuzzy_best_n_substring},
	settings::get_trigram_minimum_score,
	shell::shell_path_post_processing,
};

use crate::shell::command_output;

use tempfile::NamedTempFile;

pub fn get_error_from_multiplexer(shell: &str, command: &str) -> Option<String> {
	// terminal multiplexers, higher priority
	if let Some(error) = get_error_from_tmux(shell, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from tmux: '{}'", error);
		return Some(error);
	}
	if let Some(error) = get_error_from_screen(shell, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from screen: '{}'", error);
		return Some(error);
	}
	if let Some(error) = get_error_from_zellij(shell, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from zellij: '{}'", error);
		return Some(error);
	}

	// terminals that support capturing output
	if let Some(error) = get_error_from_kitty(shell, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from kitty: '{}'", error);
		return Some(error);
	}
	if let Some(error) = get_error_from_wezterm(shell, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from wezterm: '{}'", error);
		return Some(error);
	}
	None
}

fn get_error_from_tmux(shell: &str, command: &str) -> Option<String> {
	if std::env::var("_PR_NO_TMUX").is_ok() {
		return None;
	}
	if std::env::var("TMUX").is_err() {
		return None;
	}
	if rust_i18n::locale().to_string() != "en" {
		return None;
	}

	let capture_command = "tmux capture-pane -pS -";
	let output = command_output(shell, capture_command);

	parse_output(command, &output)
}

fn get_error_from_screen(shell: &str, command: &str) -> Option<String> {
	if std::env::var("_PR_NO_SCREEN").is_ok() {
		return None;
	}
	if std::env::var("STY").is_err() {
		return None;
	}
	if rust_i18n::locale().to_string() != "en" {
		return None;
	}

	let file = NamedTempFile::new().ok()?;
	let path = file.path().to_str()?;
	let capture_command = format!("screen -X hardcopy {}", path);
	let _ = command_output(shell, &capture_command);
	let output = std::fs::read_to_string(path).ok()?;

	parse_output(command, &output)
}

fn get_error_from_zellij(shell: &str, command: &str) -> Option<String> {
	if std::env::var("_PR_NO_ZELLIJ").is_ok() {
		return None;
	}
	if std::env::var("ZELLIJ").is_err() {
		return None;
	}
	if rust_i18n::locale().to_string() != "en" {
		return None;
	}

	let capture_command = "zellij action dump-screen --full";
	let output = command_output(shell, capture_command);

	parse_output(command, &output)
}

fn get_error_from_kitty(shell: &str, command: &str) -> Option<String> {
	if std::env::var("_PR_NO_KITTY").is_ok() {
		return None;
	}
	if std::env::var("KITTY_PID").is_err() {
		return None;
	}
	if rust_i18n::locale().to_string() != "en" {
		return None;
	}

	let capture_command = "kitty @ get-text --extent=all";
	let output = command_output(shell, capture_command);

	parse_output(command, &output)
}

fn get_error_from_wezterm(shell: &str, command: &str) -> Option<String> {
	if std::env::var("_PR_NO_WEZTERM").is_ok() {
		return None;
	}
	if std::env::var("WEZTERM_PANE").is_err() {
		return None;
	}
	if rust_i18n::locale().to_string() != "en" {
		return None;
	}

	let capture_command = "wezterm cli get-text --start-line -10000";
	let output = command_output(shell, capture_command);

	parse_output(command, &output)
}

fn parse_output(command: &str, output: &str) -> Option<String> {
	if !output.contains(command) {
		return None;
	}

	// remove everything before the last occurrence of the command
	if let Some(pos) = output.rfind(command) {
		let output = output[pos + command.len()..].trim().to_string();
		Some(output)
	} else {
		None
	}
}

pub fn zoxide_integration(
	shell: &str,
	executables: &[String],
	split: &[String],
	candidates: &mut Vec<String>,
) {
	if std::env::var("_PR_NO_ZOXIDE").is_ok() {
		return;
	}
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
