use crate::data::Data;
use pay_respects_parser::parse_rules;
use pay_respects_utils::evals::*;

#[allow(unused)]
use crate::rules_function::{Functions::*, rules_function};

pub fn match_pattern(executable: &str, data: &Data) -> Option<Vec<String>> {
	// variables to be used by parsed rules
	let error_msg = &data.error;
	let error_lower = error_msg.to_lowercase();
	let shell = &data.shell;
	let last_command = &data.command;
	let executables = &data.executables;
	let mut candidates = vec![];
	let split = split_command(last_command);

	// parse rules into rust code
	parse_rules!("rules");

	if candidates.is_empty() {
		return None;
	}
	Some(candidates)
}
