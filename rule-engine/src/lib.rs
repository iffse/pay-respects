pub mod error;
pub mod eval;
pub mod migrate;
pub mod model;
pub mod parse;
pub mod template;

pub use error::RuleError;
pub use eval::{
	Context, FunctionHandler, Mode, NoFunctions, NoScripts, ScriptHandler, evaluate_all,
	evaluate_rule,
};
pub use migrate::{migrate_file, migrate_str};
pub use model::{
	Action, Choices, CommandSlice, Condition, MatchRule, Part, RuleSet, ScriptSpec, Suggestion,
	Template,
};
pub use parse::{
	ParseOptions, parse_dir, parse_dir_with, parse_file, parse_file_with, parse_str, parse_str_with,
};

pub fn decode(bytes: &[u8]) -> Result<Vec<RuleSet>, postcard::Error> {
	postcard::from_bytes(bytes)
}
