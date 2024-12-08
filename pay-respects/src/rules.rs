use crate::shell::Data;
use pay_respects_parser::parse_rules;
use pay_respects_utils::evals::*;

pub fn match_pattern(executable: &str, data: &mut Data) {
	let error_msg = &data.error;
	let shell = &data.shell;
	let last_command = &data.command;
	let executables = &data.executables;
	let candidates = &mut data.candidates;
	parse_rules!("rules");
}
