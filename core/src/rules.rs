use crate::data::Data;
use pay_respects_parser::{parse_inline_rules, parse_rules};
use pay_respects_utils::{evals::*, modes::Mode};

#[allow(unused)]
use crate::rules_function::{Functions::*, rules_function};

pub fn match_rule(executable: &str, data: &Data) -> Option<Vec<String>> {
	use Mode::*;
	match data.mode {
		Inline => match_inline(executable, data),
		_ => match_pattern(executable, data),
	}
}

fn match_pattern(executable: &str, data: &Data) -> Option<Vec<String>> {
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

#[allow(dead_code)]
#[allow(unused)]
fn match_inline(executable: &str, data: &Data) -> Option<Vec<String>> {
	// variables to be used by parsed rules
	// error variables are not used by inlines, they are here for reusing codes
	let error_msg = &data.error;
	let error_lower = error_msg.to_lowercase();
	let shell = &data.shell;
	let last_command = &data.command;
	let executables = &data.executables;
	let mut candidates = vec![];
	let split = split_command(last_command);

	// parse rules into rust code
	parse_inline_rules!("rules");

	if candidates.is_empty() {
		return None;
	}
	Some(candidates)
}
