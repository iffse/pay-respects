pub enum Functions {
	ZoxideIntegration
}

use Functions::*;

#[allow(unused_variables)]
pub fn rules_function(
	function: Functions,
	error_msg: &str,
	error_lower: &str,
	shell: &str,
	last_command: &str,
	executables: &[String],
	split: &[String],
	) -> String {
	match function {
		ZoxideIntegration => zoxide_integration(),
	}
}

fn zoxide_integration() -> String {
	"unimplemented".to_string()
}
