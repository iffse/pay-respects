use std::sync::LazyLock;

use pay_respects_parser::parse_rules;
use pay_respects_rule_engine::{Context, Mode as RuleMode, NoScripts, RuleSet, evaluate_all};
use pay_respects_utils::evals::split_command;
use pay_respects_utils::modes::Mode;

use crate::data::Data;
use crate::rules_function::{Functions, rules_function};

static RULES: LazyLock<Vec<RuleSet>> = LazyLock::new(|| parse_rules!("rules"));

pub fn match_rule(executable: &str, data: &Data) -> Option<Vec<String>> {
	let mode = match data.mode {
		Mode::Inline => RuleMode::Inline,
		_ => RuleMode::Suggestion,
	};
	let context = Context {
		shell: &data.shell,
		shell_type: &data.shell_type,
		last_command: &data.command,
		error_msg: &data.error,
		executables: &data.executables,
	};

	let mut functions = |name: &str, _context: &Context| -> Vec<String> {
		let Some(function) = Functions::from_name(name) else {
			return Vec::new();
		};
		let error_lower = data
			.error
			.split_whitespace()
			.collect::<Vec<&str>>()
			.join("")
			.to_lowercase();
		let split = split_command(&data.command);
		let mut candidates = Vec::new();
		rules_function(
			function,
			&data.error,
			&error_lower,
			&data.shell,
			&data.command,
			&data.executables,
			&split,
			&mut candidates,
			data,
		);
		candidates
	};

	let mut scripts = NoScripts;
	let candidates = evaluate_all(
		&RULES,
		executable,
		&context,
		mode,
		&mut functions,
		&mut scripts,
	);
	if candidates.is_empty() {
		None
	} else {
		Some(candidates)
	}
}
