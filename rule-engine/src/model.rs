use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleSet {
	pub commands: Vec<String>,
	pub matches: Vec<MatchRule>,
	pub version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRule {
	pub patterns: Vec<String>,
	pub preprocess: Vec<Preprocess>,
	pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Preprocess {
	Replace {
		from: String,
		to: String,
	},
	ReplaceN {
		from: String,
		to: String,
		count: usize,
	},
}

impl Preprocess {
	pub fn apply(&self, command: &mut String) {
		match self {
			Preprocess::Replace { from, to } => *command = command.replace(from, to),
			Preprocess::ReplaceN { from, to, count } => {
				*command = command.replacen(from, to, *count);
			}
		}
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
	pub conditions: Vec<Condition>,
	pub inline: bool,
	pub action: Action,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
	Template(Template),
	Function(String),
	Script(ScriptSpec),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSpec {
	pub interpreter: String,
	pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
	pub parts: Vec<Part>,
	pub opts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Part {
	Text(String),
	CommandAll,
	Command(CommandSlice),
	Typo {
		slice: CommandSlice,
		choices: Choices,
	},
	Select {
		slice: Option<CommandSlice>,
		choices: Choices,
	},
	Selection,
	Opt(usize),
	CmdRegex(String),
	ErrRegex(String),
	Shell(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandSlice {
	Single(i32),
	Range { start: i32, end: Option<i32> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Choices {
	Literal(Vec<String>),
	Path,
	File,
	Shell(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
	Executable(String),
	CmdContains(String),
	ErrContains(String),
	Length(usize),
	MinLength(usize),
	MaxLength(usize),
	Shell(String),
	CommandSucceeds(Template),
	Not(Box<Condition>),
}

impl RuleSet {
	pub fn needs_executables(&self) -> bool {
		self.matches.iter().any(|match_rule| {
			match_rule
				.suggestions
				.iter()
				.any(Suggestion::needs_executables)
		})
	}

	pub fn has_functions(&self) -> bool {
		self.matches.iter().any(|match_rule| {
			match_rule
				.suggestions
				.iter()
				.any(|suggestion| matches!(suggestion.action, Action::Function(_)))
		})
	}
}

impl Suggestion {
	fn needs_executables(&self) -> bool {
		self.conditions.iter().any(Condition::needs_executables)
			|| match &self.action {
				Action::Template(template) => {
					template.parts.iter().any(|part| part.needs_executables())
				}
				Action::Function(_) => false,
				Action::Script(_) => true,
			}
	}
}

impl Condition {
	fn needs_executables(&self) -> bool {
		match self {
			Condition::Executable(_) => true,
			Condition::CommandSucceeds(template) => {
				template.parts.iter().any(Part::needs_executables)
			}
			Condition::Not(condition) => condition.needs_executables(),
			_ => false,
		}
	}
}

impl Part {
	fn needs_executables(&self) -> bool {
		matches!(
			self,
			Part::Typo {
				choices: Choices::Path,
				..
			} | Part::Select {
				choices: Choices::Path,
				..
			}
		)
	}
}

impl CommandSlice {
	pub fn range(&self, len: usize) -> Option<(usize, usize)> {
		let len = len as i64;
		let (start, end) = match self {
			CommandSlice::Single(index) => {
				let start = resolve_index(*index as i64, len);
				(start, start + 1)
			}
			CommandSlice::Range { start, end } => {
				let start = resolve_index(*start as i64, len);
				let end = match end {
					Some(end) => {
						let end = *end as i64;
						if end < 0 { len + end + 1 } else { end + 1 }
					}
					None => len,
				};
				(start, end)
			}
		};

		let start = start.clamp(0, len) as usize;
		let end = end.clamp(0, len) as usize;
		if start < end {
			Some((start, end))
		} else {
			None
		}
	}

	pub fn index(&self, len: usize) -> Option<usize> {
		match self {
			CommandSlice::Single(index) => {
				let index = resolve_index(*index as i64, len as i64).clamp(0, len as i64) as usize;
				if index < len { Some(index) } else { None }
			}
			CommandSlice::Range { .. } => None,
		}
	}
}

fn resolve_index(index: i64, len: i64) -> i64 {
	if index < 0 {
		(len + index).max(0)
	} else {
		index
	}
}
