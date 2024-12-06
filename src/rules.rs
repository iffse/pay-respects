use crate::shell::Data;
use crate::suggestions::*;
use pay_respects_parser::parse_rules;

pub fn match_pattern(executable: &str, data: &mut Data) {
	let error_msg = &data.error.clone();
	let shell = &data.shell.clone();
	let last_command = &data.command.clone();
	parse_rules!("rules");
}
