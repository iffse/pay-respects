use crate::suggestions::*;
use pay_respects_parser::parse_rules;

pub fn match_pattern(
	executable: &str,
	last_command: &str,
	error_msg: &str,
	shell: &str,
) -> Option<String> {
	parse_rules!("rules");
}
