use std::fmt::Write;
use std::path::Path;

use serde::Deserialize;

use crate::error::RuleError;

#[derive(Deserialize)]
struct VersionProbe {
	version: Option<u32>,
}

#[derive(Deserialize)]
struct RuleV1 {
	command: String,
	extends: Option<Vec<String>>,
	match_err: Vec<MatchV1>,
}

#[derive(Deserialize)]
struct MatchV1 {
	pattern: Option<Vec<String>>,
	suggest: Vec<String>,
}

pub fn migrate_file(path: &Path) -> Result<String, RuleError> {
	let content = std::fs::read_to_string(path)?;
	migrate_str(&content).map_err(|err| RuleError::Invalid(format!("{}: {}", path.display(), err)))
}

pub fn migrate_str(content: &str) -> Result<String, RuleError> {
	let probe: VersionProbe = toml::from_str(content)?;
	if probe.version == Some(2) {
		return Err(RuleError::Invalid(
			"rule file is already version 2".to_string(),
		));
	}
	if probe.version.is_some_and(|version| version != 1) {
		return Err(RuleError::Invalid(format!(
			"unsupported rule version: {}",
			probe.version.unwrap()
		)));
	}

	let rule: RuleV1 = toml::from_str(content)?;
	let mut output = String::new();

	writeln!(output, "version = 2").unwrap();
	let mut commands = vec![rule.command];
	commands.extend(rule.extends.unwrap_or_default());
	write_array(&mut output, "commands", &commands);

	for match_err in rule.match_err {
		writeln!(output).unwrap();
		writeln!(output, "[[match]]").unwrap();
		if let Some(patterns) = match_err.pattern
			&& !patterns.is_empty()
		{
			write_array(&mut output, "error", &patterns);
		}

		for suggestion in match_err.suggest {
			write_suggestion(&mut output, &suggestion)?;
		}
	}

	Ok(output)
}

fn write_suggestion(output: &mut String, suggestion: &str) -> Result<(), RuleError> {
	let (body, conditions) = split_conditions(suggestion)?;
	let mut conditions = conditions.unwrap_or_default();

	let has_inline = conditions.iter().any(|condition| condition == "INLINE");
	let has_function = conditions.iter().any(|condition| condition == "FUNCTION");
	conditions.retain(|condition| condition != "INLINE" && condition != "FUNCTION");

	writeln!(output).unwrap();
	writeln!(output, "[[match.suggest]]").unwrap();
	if !conditions.is_empty() {
		write_array(output, "when", &conditions);
	}
	if has_inline {
		writeln!(output, "inline = true").unwrap();
	}
	if has_function {
		writeln!(output, "function = {}", toml_string(body.trim())).unwrap();
	} else {
		writeln!(output, "run = {}", toml_template(body.trim())).unwrap();
	}

	Ok(())
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
	let conditions = split_unescaped_commas(conditions);
	let body = lines[body_start..].join("\n");
	Ok((body, Some(conditions)))
}

fn split_unescaped_commas(input: &str) -> Vec<String> {
	let mut result = Vec::new();
	let mut current = String::new();
	let mut backslashes = 0;

	for character in input.chars() {
		if character == ',' && backslashes % 2 == 0 {
			result.push(current.trim().to_string());
			current.clear();
		} else {
			current.push(character);
		}
		if character == '\\' {
			backslashes += 1;
		} else {
			backslashes = 0;
		}
	}
	result.push(current.trim().to_string());
	result
		.into_iter()
		.filter(|condition| !condition.is_empty())
		.collect()
}

fn write_array(output: &mut String, name: &str, values: &[String]) {
	writeln!(output, "{} = [", name).unwrap();
	for value in values {
		writeln!(output, "\t{},", toml_string(value)).unwrap();
	}
	writeln!(output, "]").unwrap();
}

fn toml_string(value: &str) -> String {
	if !value.contains('\'') && !value.contains('\n') && !value.contains('\r') {
		return format!("'{}'", value);
	}

	let mut output = String::from("\"");
	for character in value.chars() {
		match character {
			'"' => output.push_str("\\\""),
			'\\' => output.push_str("\\\\"),
			'\n' => output.push_str("\\n"),
			'\r' => output.push_str("\\r"),
			'\t' => output.push_str("\\t"),
			character if character.is_control() => {
				write!(output, "\\u{:04X}", character as u32).unwrap();
			}
			character => output.push(character),
		}
	}
	output.push('"');
	output
}

fn toml_template(value: &str) -> String {
	if value.contains("'''") {
		return toml_string(value);
	}
	format!("'''{}'''", value)
}
