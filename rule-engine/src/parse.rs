use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use pay_respects_utils::strings::split_unescaped_character;
use serde::Deserialize;

use crate::error::RuleError;
use crate::model::{Action, Condition, MatchRule, Preprocess, RuleSet, ScriptSpec, Suggestion};
use crate::template;

#[derive(Clone, Copy)]
pub struct ParseOptions {
	pub allow_scripts: bool,
}

impl Default for ParseOptions {
	fn default() -> Self {
		Self {
			allow_scripts: true,
		}
	}
}

#[derive(Deserialize)]
struct VersionProbe {
	version: Option<u32>,
}

#[derive(Deserialize)]
struct RuleFileV1 {
	command: String,
	extends: Option<Vec<String>>,
	match_err: Vec<MatchErrorV1>,
}

#[derive(Deserialize)]
struct MatchErrorV1 {
	pattern: Option<Vec<String>>,
	suggest: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleFileV2 {
	version: u32,
	commands: Vec<String>,
	#[serde(default, rename = "match")]
	matches: Vec<MatchV2>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MatchV2 {
	error: Option<Vec<String>>,
	vars: Option<VarsV2>,
	#[serde(default)]
	preprocess: Vec<String>,
	#[serde(default)]
	suggest: Vec<SuggestionV2>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum VarsV2 {
	Map(HashMap<String, String>),
	Pairs(Vec<(String, String)>),
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SuggestionV2 {
	#[serde(default)]
	when: Vec<String>,
	#[serde(default)]
	inline: bool,
	function: Option<String>,
	script: Option<ScriptSpecV2>,
	run: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScriptSpecV2 {
	interpreter: String,
	file: String,
}

pub fn parse_str(content: &str) -> Result<RuleSet, RuleError> {
	parse_str_with(content, ParseOptions::default())
}

pub fn parse_str_with(content: &str, options: ParseOptions) -> Result<RuleSet, RuleError> {
	let probe: VersionProbe = toml::from_str(content)?;
	match probe.version {
		None | Some(1) => parse_v1(content),
		Some(2) => parse_v2(content, options),
		Some(version) => Err(RuleError::Invalid(format!(
			"unsupported rule version: {}",
			version
		))),
	}
}

pub fn parse_file(path: &Path) -> Result<RuleSet, RuleError> {
	parse_file_with(path, ParseOptions::default())
}

pub fn parse_file_with(path: &Path, options: ParseOptions) -> Result<RuleSet, RuleError> {
	let content = std::fs::read_to_string(path)?;
	parse_str_with(&content, options)
		.map_err(|err| RuleError::Invalid(format!("{}: {}", path.display(), err)))
}

pub fn parse_dir(path: &Path) -> Result<Vec<RuleSet>, RuleError> {
	parse_dir_with(path, ParseOptions::default())
}

pub fn parse_dir_with(path: &Path, options: ParseOptions) -> Result<Vec<RuleSet>, RuleError> {
	let mut files = std::fs::read_dir(path)?
		.filter_map(|entry| entry.ok())
		.map(|entry| entry.path())
		.filter(|path| path.extension().is_some_and(|ext| ext == "toml"))
		.collect::<Vec<PathBuf>>();
	files.sort();

	files
		.iter()
		.map(|path| parse_file_with(path, options))
		.collect()
}

fn parse_v1(content: &str) -> Result<RuleSet, RuleError> {
	let file: RuleFileV1 = toml::from_str(content)?;
	let mut commands = vec![file.command];
	commands.extend(file.extends.unwrap_or_default());
	let commands = normalize_commands(commands)?;

	let matches = file
		.match_err
		.into_iter()
		.map(|match_err| {
			let suggestions = match_err
				.suggest
				.into_iter()
				.map(|suggestion| parse_v1_suggestion(&suggestion))
				.collect::<Result<Vec<Suggestion>, RuleError>>()?;
			Ok(MatchRule {
				patterns: normalize_patterns(match_err.pattern),
				preprocess: Vec::new(),
				suggestions,
			})
		})
		.collect::<Result<Vec<MatchRule>, RuleError>>()?;

	Ok(RuleSet {
		commands,
		matches,
		version: 1,
	})
}

fn parse_v2(content: &str, options: ParseOptions) -> Result<RuleSet, RuleError> {
	let file: RuleFileV2 = toml::from_str(content)?;
	debug_assert_eq!(file.version, 2);
	let commands = normalize_commands(file.commands)?;

	let matches = file
		.matches
		.into_iter()
		.map(|match_rule| {
			let vars = build_vars(match_rule.vars)?;
			let preprocess = match_rule
				.preprocess
				.into_iter()
				.map(|operation| {
					let operation = expand_vars(&operation, &vars)?;
					parse_preprocess(&operation)
				})
				.collect::<Result<Vec<Preprocess>, RuleError>>()?;
			let suggestions = match_rule
				.suggest
				.into_iter()
				.map(|suggestion| parse_v2_suggestion(suggestion, &vars, options))
				.collect::<Result<Vec<Suggestion>, RuleError>>()?;
			Ok(MatchRule {
				patterns: normalize_patterns(match_rule.error),
				preprocess,
				suggestions,
			})
		})
		.collect::<Result<Vec<MatchRule>, RuleError>>()?;

	Ok(RuleSet {
		commands,
		matches,
		version: 2,
	})
}

fn build_vars(vars: Option<VarsV2>) -> Result<HashMap<String, String>, RuleError> {
	let mut parsed = HashMap::new();

	let entries = match vars {
		Some(VarsV2::Map(vars)) => vars.into_iter().collect::<Vec<(String, String)>>(),
		Some(VarsV2::Pairs(vars)) => vars,
		None => return Ok(parsed),
	};

	for (name, value) in entries {
		let name = name.trim().trim_start_matches('$').to_string();
		if name.is_empty() {
			return Err(RuleError::Invalid(
				"variable name cannot be empty".to_string(),
			));
		}
		if parsed.insert(name.clone(), value).is_some() {
			return Err(RuleError::Invalid(format!("duplicate variable: {}", name)));
		}
	}

	Ok(parsed)
}

fn parse_preprocess(input: &str) -> Result<Preprocess, RuleError> {
	let input = input.trim();
	let open = input
		.find('(')
		.ok_or_else(|| RuleError::Invalid(format!("preprocess is missing `(`: {}", input)))?;
	let close = input
		.rfind(')')
		.ok_or_else(|| RuleError::Invalid(format!("preprocess is missing `)`: {}", input)))?;
	if close <= open {
		return Err(RuleError::Invalid(format!(
			"preprocess has an invalid argument list: {}",
			input
		)));
	}

	let name = input[..open].trim();
	let args = split_preprocess_args(&input[open + 1..close])?;

	match name {
		"replace" => {
			if args.len() != 2 {
				return Err(RuleError::Invalid(format!(
					"replace expects 2 arguments: {}",
					input
				)));
			}
			Ok(Preprocess::Replace {
				from: args[0].clone(),
				to: args[1].clone(),
			})
		}
		"replacen" => {
			if args.len() != 3 {
				return Err(RuleError::Invalid(format!(
					"replacen expects 3 arguments: {}",
					input
				)));
			}
			let count = args[2]
				.parse::<usize>()
				.map_err(|_| RuleError::Invalid(format!("invalid replacen count: {}", args[2])))?;
			Ok(Preprocess::ReplaceN {
				from: args[0].clone(),
				to: args[1].clone(),
				count,
			})
		}
		_ => Err(RuleError::Invalid(format!("unknown preprocess: {}", name))),
	}
}

fn split_preprocess_args(input: &str) -> Result<Vec<String>, RuleError> {
	let mut args = Vec::new();
	let mut current = String::new();
	let mut quote = None;
	let mut quoted = false;
	let mut after_quote = false;
	let mut escaped = false;
	let mut any = false;

	for character in input.chars() {
		any = true;
		if escaped {
			match character {
				'n' => current.push('\n'),
				'r' => current.push('\r'),
				't' => current.push('\t'),
				'\\' => current.push('\\'),
				'\'' => current.push('\''),
				'"' => current.push('"'),
				',' => current.push(','),
				character => {
					current.push('\\');
					current.push(character);
				}
			}
			after_quote = false;
			escaped = false;
			continue;
		}

		if character == '\\' {
			escaped = true;
			continue;
		}

		if let Some(delimiter) = quote {
			if character == delimiter {
				quote = None;
				after_quote = true;
			} else {
				current.push(character);
			}
			continue;
		}

		if character.is_whitespace() && after_quote {
			continue;
		}

		match character {
			'\'' | '"' => {
				if !quoted && current.trim().is_empty() {
					current.clear();
				}
				quote = Some(character);
				quoted = true;
				after_quote = false;
			}
			',' => {
				push_preprocess_arg(&mut args, &mut current, quoted);
				quoted = false;
				after_quote = false;
			}
			character => {
				current.push(character);
				after_quote = false;
			}
		}
	}

	if escaped {
		current.push('\\');
	}
	if quote.is_some() {
		return Err(RuleError::Invalid(format!(
			"unclosed quote in preprocess: {}",
			input
		)));
	}
	if any {
		push_preprocess_arg(&mut args, &mut current, quoted);
	}

	Ok(args)
}

fn push_preprocess_arg(args: &mut Vec<String>, current: &mut String, quoted: bool) {
	let arg = if quoted {
		current.clone()
	} else {
		current.trim().to_string()
	};
	args.push(arg);
	current.clear();
}

fn parse_v1_suggestion(input: &str) -> Result<Suggestion, RuleError> {
	let (body, conditions) = split_conditions(input)?;
	let (conditions, inline, function) = parse_conditions(conditions)?;

	let action = if function {
		Action::Function(body.trim().to_string())
	} else {
		Action::Template(template::parse(&body)?)
	};

	Ok(Suggestion {
		conditions,
		inline,
		action,
	})
}

fn parse_v2_suggestion(
	suggestion: SuggestionV2,
	vars: &HashMap<String, String>,
	options: ParseOptions,
) -> Result<Suggestion, RuleError> {
	let (conditions, inline, is_function) = parse_conditions(Some(suggestion.when))?;
	if is_function {
		return Err(RuleError::Invalid(
			"use the `function` field instead of `FUNCTION` in `when`".to_string(),
		));
	}
	let inline = inline || suggestion.inline;

	if suggestion.script.is_some() && !options.allow_scripts {
		return Err(RuleError::Invalid(
			"script suggestions are only available in runtime rules".to_string(),
		));
	}

	let action = match (suggestion.run, suggestion.function, suggestion.script) {
		(Some(_), Some(_), _) | (Some(_), _, Some(_)) | (_, Some(_), Some(_)) => {
			return Err(RuleError::Invalid(
				"suggestion must have exactly one of `run`, `function`, or `script`".to_string(),
			));
		}
		(Some(run), None, None) => {
			let run = expand_vars(&run, vars)?;
			Action::Template(template::parse(&run)?)
		}
		(None, Some(function), None) => Action::Function(function),
		(None, None, Some(script)) => Action::Script(ScriptSpec {
			interpreter: expand_vars(&script.interpreter, vars)?,
			file: expand_vars(&script.file, vars)?,
		}),
		(None, None, None) => {
			return Err(RuleError::Invalid(
				"suggestion must have one of `run`, `function`, or `script`".to_string(),
			));
		}
	};

	Ok(Suggestion {
		conditions,
		inline,
		action,
	})
}

fn expand_vars(input: &str, vars: &HashMap<String, String>) -> Result<String, RuleError> {
	expand_vars_inner(input, vars, &mut Vec::new())
}

fn expand_vars_inner(
	input: &str,
	vars: &HashMap<String, String>,
	stack: &mut Vec<String>,
) -> Result<String, RuleError> {
	let mut result = String::new();
	let mut rest = input;

	while let Some(start) = rest.find("{{$") {
		result.push_str(&rest[..start]);
		let after = &rest[start + 3..];
		let end = after
			.find("}}")
			.ok_or_else(|| RuleError::Invalid("unclosed variable placeholder".to_string()))?;
		let name = after[..end].trim();
		if name.is_empty() {
			return Err(RuleError::Invalid(
				"variable name cannot be empty".to_string(),
			));
		}
		if stack.iter().any(|item| item == name) {
			let mut cycle = stack.join(" -> ");
			if !cycle.is_empty() {
				cycle.push_str(" -> ");
			}
			cycle.push_str(name);
			return Err(RuleError::Invalid(format!("recursive variable: {}", cycle)));
		}
		let value = vars
			.get(name)
			.ok_or_else(|| RuleError::Invalid(format!("unknown variable: {}", name)))?;
		stack.push(name.to_string());
		result.push_str(&expand_vars_inner(value, vars, stack)?);
		stack.pop();
		rest = &after[end + 2..];
	}

	result.push_str(rest);
	Ok(result)
}

fn split_conditions(input: &str) -> Result<(String, Option<Vec<String>>), RuleError> {
	if !input.starts_with('#') {
		return Ok((input.to_string(), None));
	}

	let lines = input.lines().collect::<Vec<&str>>();
	let mut conditions = String::new();
	let mut body_start = None;

	for (index, line) in lines.iter().enumerate() {
		conditions.push_str(line);
		if line.ends_with(']') {
			body_start = Some(index + 1);
			break;
		}
	}

	let body_start = body_start
		.ok_or_else(|| RuleError::Invalid("unclosed condition header in suggestion".to_string()))?;
	let conditions = conditions
		.trim_start_matches(['#', '['])
		.trim_end_matches(']');
	let conditions = split_unescaped_character(conditions, ',')
		.into_iter()
		.map(|condition| condition.trim().to_string())
		.collect();

	Ok((lines[body_start..].join("\n"), Some(conditions)))
}

fn parse_conditions(
	conditions: Option<Vec<String>>,
) -> Result<(Vec<Condition>, bool, bool), RuleError> {
	let mut parsed = Vec::new();
	let mut inline = false;
	let mut function = false;

	for condition in conditions.unwrap_or_default() {
		let condition = condition.trim();
		if condition.is_empty() {
			continue;
		}
		match condition {
			"INLINE" => inline = true,
			"FUNCTION" => function = true,
			_ => parsed.push(parse_condition(condition)?),
		}
	}

	Ok((parsed, inline, function))
}

fn parse_condition(input: &str) -> Result<Condition, RuleError> {
	let (negated, input) = match input.strip_prefix('!') {
		Some(input) => (true, input),
		None => (false, input),
	};
	let (name, arg) = input.split_once('(').ok_or_else(|| {
		RuleError::Invalid(format!("condition is missing an argument: {}", input))
	})?;
	let arg = arg.strip_suffix(')').ok_or_else(|| {
		RuleError::Invalid(format!("condition has an unclosed argument: {}", input))
	})?;

	let condition = match name.trim() {
		"executable" => Condition::Executable(arg.to_string()),
		"cmd_contains" => Condition::CmdContains(arg.to_string()),
		"err_contains" => Condition::ErrContains(arg.to_string()),
		"length" => Condition::Length(parse_usize(arg, "length")?),
		"min_length" => Condition::MinLength(parse_usize(arg, "min_length")?),
		"max_length" => Condition::MaxLength(parse_usize(arg, "max_length")?),
		"shell" => Condition::Shell(arg.to_string()),
		"command_succeeds" => Condition::CommandSucceeds(template::parse(arg)?),
		_ => {
			return Err(RuleError::Invalid(format!(
				"unknown condition: {}",
				name.trim()
			)));
		}
	};

	Ok(if negated {
		Condition::Not(Box::new(condition))
	} else {
		condition
	})
}

fn parse_usize(input: &str, condition: &str) -> Result<usize, RuleError> {
	input
		.trim()
		.parse::<usize>()
		.map_err(|_| RuleError::Invalid(format!("invalid number for `{}`: {}", condition, input)))
}

fn normalize_patterns(patterns: Option<Vec<String>>) -> Vec<String> {
	match patterns {
		Some(patterns) if !patterns.is_empty() => patterns
			.into_iter()
			.map(|pattern| normalize_pattern(&pattern))
			.collect(),
		_ => vec![String::new()],
	}
}

pub fn normalize_pattern(pattern: &str) -> String {
	pattern
		.split_whitespace()
		.collect::<Vec<&str>>()
		.join("")
		.to_lowercase()
}

fn normalize_commands(commands: Vec<String>) -> Result<Vec<String>, RuleError> {
	let mut seen = HashSet::new();
	let mut normalized = Vec::new();

	for command in commands {
		let command = command.trim().to_string();
		if command.is_empty() {
			return Err(RuleError::Invalid(
				"command name cannot be empty".to_string(),
			));
		}
		if seen.insert(command.clone()) {
			normalized.push(command);
		}
	}

	if normalized.is_empty() {
		return Err(RuleError::Invalid("no commands defined".to_string()));
	}

	Ok(normalized)
}
