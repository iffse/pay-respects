use pay_respects_utils::evals::{
	best_matches, cmd_regex, command_succeeds, err_regex, eval_shell_command, opt_regex,
	regex_match, regex_match_spaceless, split_command, suggest_typo,
};
use pay_respects_utils::files::best_match_file;

use crate::model::{Action, Choices, CommandSlice, Condition, Part, RuleSet, ScriptSpec, Template};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
	Suggestion,
	Inline,
}

pub struct Context<'a> {
	pub shell: &'a str,
	pub shell_type: &'a str,
	pub last_command: &'a str,
	pub error_msg: &'a str,
	pub executables: &'a [String],
}

pub trait FunctionHandler {
	fn evaluate(&mut self, name: &str, context: &Context) -> Vec<String>;
}

pub struct NoFunctions;

impl FunctionHandler for NoFunctions {
	fn evaluate(&mut self, _name: &str, _context: &Context) -> Vec<String> {
		Vec::new()
	}
}

impl<F> FunctionHandler for F
where
	F: FnMut(&str, &Context) -> Vec<String>,
{
	fn evaluate(&mut self, name: &str, context: &Context) -> Vec<String> {
		self(name, context)
	}
}

pub trait ScriptHandler {
	fn evaluate(&mut self, script: &ScriptSpec, context: &Context) -> Vec<String>;
}

pub struct NoScripts;

impl ScriptHandler for NoScripts {
	fn evaluate(&mut self, _script: &ScriptSpec, _context: &Context) -> Vec<String> {
		Vec::new()
	}
}

pub fn evaluate_all<F: FunctionHandler, S: ScriptHandler>(
	rules: &[RuleSet],
	command: &str,
	context: &Context,
	mode: Mode,
	functions: &mut F,
	scripts: &mut S,
) -> Vec<String> {
	let mut candidates = Vec::new();

	for rule in rules {
		if rule
			.commands
			.iter()
			.any(|rule_command| rule_command == command)
		{
			candidates.extend(evaluate_rule(rule, context, mode, functions, scripts));
		}
	}

	candidates
}

pub fn evaluate_rule<F: FunctionHandler, S: ScriptHandler>(
	rule: &RuleSet,
	context: &Context,
	mode: Mode,
	functions: &mut F,
	scripts: &mut S,
) -> Vec<String> {
	let error = normalize_error(context.error_msg);
	let mut candidates = Vec::new();

	for match_rule in &rule.matches {
		if mode == Mode::Suggestion
			&& !match_rule
				.patterns
				.iter()
				.any(|pattern| error.contains(pattern))
		{
			continue;
		}

		let mut command = context.last_command.to_string();
		for operation in &match_rule.preprocess {
			operation.apply(&mut command);
		}
		let context = Context {
			shell: context.shell,
			shell_type: context.shell_type,
			last_command: &command,
			error_msg: context.error_msg,
			executables: context.executables,
		};
		let split = split_command(context.last_command);

		for suggestion in &match_rule.suggestions {
			if mode == Mode::Inline && !suggestion.inline {
				continue;
			}
			if !conditions_match(&suggestion.conditions, &context, mode, &error, &split) {
				continue;
			}

			match &suggestion.action {
				Action::Template(template) => {
					candidates.extend(render_template(template, &context));
				}
				Action::Function(name) => {
					candidates.extend(functions.evaluate(name, &context));
				}
				Action::Script(script) => {
					candidates.extend(scripts.evaluate(script, &context));
				}
			}
		}
	}

	candidates
}

fn conditions_match(
	conditions: &[Condition],
	context: &Context,
	mode: Mode,
	error: &str,
	split: &[String],
) -> bool {
	conditions
		.iter()
		.all(|condition| condition_match(condition, context, mode, error, split))
}

fn condition_match(
	condition: &Condition,
	context: &Context,
	mode: Mode,
	error: &str,
	split: &[String],
) -> bool {
	if mode == Mode::Inline && is_error_condition(condition) {
		return true;
	}

	match condition {
		Condition::Executable(executable) => {
			context.executables.iter().any(|item| item == executable)
		}
		Condition::CmdContains(regex) => regex_match(regex, context.last_command),
		Condition::ErrContains(regex) => regex_match_spaceless(regex, error),
		Condition::Length(length) => split.len() == *length,
		Condition::MinLength(length) => split.len() >= *length,
		Condition::MaxLength(length) => split.len() <= *length + 1,
		Condition::Shell(shell) => context.shell_type == shell,
		Condition::CommandSucceeds(template) => render_template(template, context)
			.iter()
			.any(|command| !command.trim().is_empty() && command_succeeds(context.shell, command)),
		Condition::Not(condition) => !condition_match(condition, context, mode, error, split),
	}
}

fn is_error_condition(condition: &Condition) -> bool {
	match condition {
		Condition::ErrContains(_) => true,
		Condition::Not(condition) => matches!(condition.as_ref(), Condition::ErrContains(_)),
		_ => false,
	}
}

fn render_template(template: &Template, context: &Context) -> Vec<String> {
	let mut command = context.last_command.to_string();
	let mut opt_values = Vec::with_capacity(template.opts.len());

	for regex in &template.opts {
		let captures = opt_regex(regex, &mut command);
		opt_values.push(if captures.is_empty() {
			String::new()
		} else {
			format!(" {}", captures)
		});
	}

	let split = split_command(&command);
	let select = template.parts.iter().find_map(|part| match part {
		Part::Select { slice, choices } => Some((slice, choices)),
		_ => None,
	});

	let selections = if let Some((slice, choices)) = select {
		evaluate_select(slice, choices, &split, context)
	} else {
		Vec::new()
	};

	let render = |selection: Option<&str>| {
		let mut result = String::new();

		for part in &template.parts {
			match part {
				Part::Text(text) => result.push_str(text),
				Part::CommandAll => result.push_str(&command),
				Part::Command(slice) => {
					if let Some((start, end)) = slice.range(split.len()) {
						result.push_str(&split[start..end].join(" "));
					}
				}
				Part::Typo { slice, choices } => {
					if let Some((start, end)) = slice.range(split.len()) {
						let choices = resolve_choices(choices, context);
						result.push_str(&suggest_typo(
							&split[start..end],
							&choices,
							context.executables,
						));
					}
				}
				Part::Select { .. } | Part::Selection => {
					if let Some(selection) = selection {
						result.push_str(selection);
					}
				}
				Part::Opt(index) => result.push_str(&opt_values[*index]),
				Part::CmdRegex(regex) => result.push_str(&cmd_regex(regex, &command)),
				Part::ErrRegex(regex) => result.push_str(&err_regex(regex, context.error_msg)),
				Part::Shell(command) => {
					result.push_str(&eval_shell_command(context.shell, command).join(""));
				}
			}
		}

		result
	};

	if select.is_some() {
		selections
			.iter()
			.map(|selection| render(Some(selection)))
			.collect()
	} else {
		vec![render(None)]
	}
}

fn evaluate_select(
	slice: &Option<CommandSlice>,
	choices: &Choices,
	split: &[String],
	context: &Context,
) -> Vec<String> {
	match choices {
		Choices::Literal(values) => values.clone(),
		Choices::Path => {
			let index = match slice {
				Some(slice) => slice.index(split.len()),
				None => (!split.is_empty()).then_some(0),
			};
			let Some(index) = index else {
				return Vec::new();
			};
			best_matches(&split[index], context.executables)
				.unwrap_or_else(|| vec![split[index].clone()])
		}
		Choices::File => {
			let index = match slice {
				Some(slice) => slice.index(split.len()),
				None => (!split.is_empty()).then_some(0),
			};
			let Some(index) = index else {
				return Vec::new();
			};
			match best_match_file(&split[index]) {
				Some(file) => vec![file],
				None => vec![split[index].clone()],
			}
		}
		Choices::Shell(command) => eval_shell_command(context.shell, command),
	}
}

fn resolve_choices(choices: &Choices, context: &Context) -> Vec<String> {
	match choices {
		Choices::Literal(values) => values.clone(),
		Choices::Path => vec!["path".to_string()],
		Choices::File => vec!["file".to_string()],
		Choices::Shell(command) => eval_shell_command(context.shell, command),
	}
}

pub fn normalize_error(error: &str) -> String {
	error
		.split_whitespace()
		.collect::<Vec<&str>>()
		.join("")
		.to_lowercase()
}
