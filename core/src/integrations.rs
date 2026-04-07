use pay_respects_utils::{
	evals::{compare_string, fuzzy_best_n_substring},
	settings::get_trigram_minimum_score,
	shell::shell_path_post_processing,
	strings::{print_error, remove_color_codes},
};

use crate::shell::command_output;

use tempfile::NamedTempFile;

#[allow(unreachable_code)]
#[allow(unused_variables)]
pub fn get_error_from_multiplexer(
	shell: &str,
	prompt_prefix: &Option<String>,
	command: &str,
) -> Option<String> {
	// in debug mode the output is not clear due to logs
	#[cfg(debug_assertions)]
	{
		return None;
	}

	if std::env::var("_PR_NO_MULTIPLEXER").is_ok() {
		return None;
	}

	let prefix = if let Some(prefix) = prompt_prefix {
		prefix
	} else {
		return None;
	};

	// terminal multiplexers, higher priority
	if let Some(error) = get_error_from_tmux(shell, prefix, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from tmux: '{}'", error);
		return Some(error);
	}
	if let Some(error) = get_error_from_screen(shell, prefix, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from screen: '{}'", error);
		return Some(error);
	}
	if let Some(error) = get_error_from_zellij(shell, prefix, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from zellij: '{}'", error);
		return Some(error);
	}

	// terminals that support capturing output
	if let Some(error) = get_error_from_kitty(shell, prefix, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from kitty: '{}'", error);
		return Some(error);
	}
	if let Some(error) = get_error_from_wezterm(shell, prefix, command) {
		#[cfg(debug_assertions)]
		eprintln!("Error captured from wezterm: '{}'", error);
		return Some(error);
	}
	None
}

fn get_error_from_tmux(shell: &str, prefix: &str, command: &str) -> Option<String> {
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

	parse_output(prefix, command, &output)
}

fn get_error_from_screen(shell: &str, prefix: &str, command: &str) -> Option<String> {
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

	parse_output(prefix, command, &output)
}

fn get_error_from_zellij(shell: &str, prefix: &str, command: &str) -> Option<String> {
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

	parse_output(prefix, command, &output)
}

fn get_error_from_kitty(shell: &str, prefix: &str, command: &str) -> Option<String> {
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

	parse_output(prefix, command, &output)
}

fn get_error_from_wezterm(shell: &str, prefix: &str, command: &str) -> Option<String> {
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

	parse_output(prefix, command, &output)
}

fn parse_output(prefix: &str, input_command: &str, capture: &str) -> Option<String> {
	// no space and newline, to make it more robust
	let command = input_command.replace(|c: char| c.is_whitespace(), "");
	let prefix = remove_color_codes(prefix);

	let mut start_pos = capture.rfind(&prefix)?;
	let mut end_pos = capture.len();
	loop {
		let tail =
			capture[start_pos + prefix.len()..end_pos].replace(|c: char| c.is_whitespace(), "");
		if tail.starts_with(&command) {
			let cmderr = capture[start_pos + prefix.len()..].trim();
			let error = cmderr[input_command.trim_start().len()..]
				.trim()
				.to_string();
			return Some(error);
		} else if let Some(pos) = capture[..start_pos].rfind(&prefix) {
			end_pos = start_pos;
			start_pos = pos;
		} else {
			print_error("Failed to capture output from multiplexer.");
			break;
		}
	}
	None
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

	let current_dir = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(""));

	let joined_hints = hints.join(" ");
	if !filtered_directories.is_empty() {
		let mut min_distance = usize::MAX;
		let mut bests = Vec::new();

		for directory in filtered_directories {
			let distance = compare_string(&joined_hints, directory);
			if distance < min_distance {
				min_distance = distance;
				bests.clear();
				bests.push(directory.to_string());
			} else if distance == min_distance {
				bests.push(directory.to_string());
			}
		}

		let mut valid = false;
		for directory in bests {
			if directory == current_dir.to_str().unwrap() {
				continue;
			}
			valid = true;
			let directory = shell_path_post_processing(&directory);
			candidates.push(format!("cd {}", directory.clone()));
		}
		if valid {
			return;
		}
	}

	let match_candidates = directories.map(|s| s.to_string()).collect::<Vec<String>>();
	let directories = fuzzy_best_n_substring(
		&joined_hints,
		&match_candidates,
		get_trigram_minimum_score(),
		3,
	);
	if let Some(directories) = directories {
		for directory in directories {
			if directory == current_dir.to_str().unwrap() {
				continue;
			}
			let directory = shell_path_post_processing(&directory);
			candidates.push(format!("cd {}", directory.clone()));
		}
	}
}
