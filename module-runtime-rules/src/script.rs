use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use pay_respects_rule_engine::{Context, ScriptHandler, ScriptSpec};
use pay_respects_utils::strings::print_error;

pub struct ExternalScripts {
	script_dirs: Vec<PathBuf>,
	rule_dir: PathBuf,
}

impl ExternalScripts {
	pub fn new(rule_dir: &Path, script_dirs: &[PathBuf]) -> Self {
		Self {
			script_dirs: script_dirs.to_vec(),
			rule_dir: rule_dir.to_path_buf(),
		}
	}

	fn resolve(&self, file: &str) -> Option<PathBuf> {
		let path = Path::new(file);
		if path.is_absolute() {
			return path.is_file().then(|| path.to_path_buf());
		}

		for dir in &self.script_dirs {
			let path = dir.join(file);
			if path.is_file() {
				return Some(path);
			}
		}

		let path = self.rule_dir.join(file);
		path.is_file().then_some(path)
	}
}

impl ScriptHandler for ExternalScripts {
	fn evaluate(&mut self, script: &ScriptSpec, context: &Context) -> Vec<String> {
		let Some(file) = self.resolve(&script.file) else {
			print_error(&format!("Script not found: {}", script.file));
			return Vec::new();
		};

		let executables = {
			let executables = context.executables.join(" ");
			if executables.len() < 100_000 {
				executables
			} else {
				String::new()
			}
		};
		let command = context.last_command.split_whitespace().next().unwrap_or("");

		let mut process = Command::new(&script.interpreter);
		process
			.arg(&file)
			.env("_PR_COMMAND", command)
			.env("_PR_LAST_COMMAND", context.last_command)
			.env("_PR_ERROR_MSG", context.error_msg)
			.env("_PR_SHELL", context.shell)
			.env("_PR_EXECUTABLES", executables)
			.stdin(Stdio::null());

		let output = match process.output() {
			Ok(output) => output,
			Err(err) => {
				print_error(&format!("Failed to run {}: {}", script.interpreter, err));
				return Vec::new();
			}
		};

		if !output.status.success() {
			let stderr = String::from_utf8_lossy(&output.stderr);
			if !stderr.trim().is_empty() {
				print_error(&format!("Script failed: {}", stderr.trim()));
			}
		}

		parse_output(&String::from_utf8_lossy(&output.stdout))
	}
}

fn parse_output(output: &str) -> Vec<String> {
	if output.contains("<_PR_BR>") {
		return output
			.split("<_PR_BR>")
			.map(str::trim)
			.filter(|suggestion| !suggestion.is_empty())
			.map(str::to_string)
			.collect();
	}

	output
		.lines()
		.map(str::trim)
		.filter(|suggestion| !suggestion.is_empty())
		.map(str::to_string)
		.collect()
}
