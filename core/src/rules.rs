use crate::shell::Data;
use pay_respects_parser::parse_rules;
use pay_respects_utils::evals::*;

pub fn match_pattern(executable: &str, data: &Data) -> Option<Vec<String>> {
	let error_msg = &data.error;
	let shell = &data.shell;
	let last_command = &data.command;
	let executables = &data.executables;
	let mut candidates = vec![];
	let split = split_command(last_command);
	parse_rules!("rules");
	if candidates.is_empty() {
		None
	} else {
		Some(candidates)
	}
}
