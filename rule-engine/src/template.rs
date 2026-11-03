use crate::error::RuleError;
use crate::model::{Choices, CommandSlice, Part, Template};

enum ParsedPart {
	Part(Part),
	Opt(String),
}

impl From<Part> for ParsedPart {
	fn from(part: Part) -> Self {
		ParsedPart::Part(part)
	}
}

pub fn parse(input: &str) -> Result<Template, RuleError> {
	let mut parts = Vec::new();
	let mut opts = Vec::new();
	let mut rest = input;

	while let Some(start) = rest.find("{{") {
		let (before, placeholder) = rest.split_at(start);
		if !before.is_empty() {
			parts.push(Part::Text(before.to_string()));
		}

		let placeholder = &placeholder[2..];
		let end = find_end(placeholder)?;
		let body = &placeholder[..end];

		match parse_part(body)? {
			ParsedPart::Opt(regex) => {
				if let Some(Part::Text(text)) = parts.last_mut()
					&& text.ends_with(' ')
				{
					text.pop();
				}
				let index = opts.len();
				opts.push(regex);
				parts.push(Part::Opt(index));
			}
			ParsedPart::Part(part) => parts.push(part),
		}

		rest = &placeholder[end + 2..];
	}

	if !rest.is_empty() {
		parts.push(Part::Text(rest.to_string()));
	}

	if parts
		.iter()
		.filter(|part| matches!(part, Part::Select { .. }))
		.count()
		> 1
	{
		return Err(RuleError::Invalid(
			"only one {{select}} placeholder is supported".to_string(),
		));
	}

	Ok(Template { parts, opts })
}

fn find_end(input: &str) -> Result<usize, RuleError> {
	let bytes = input.as_bytes();
	let mut depth = 0;
	let mut index = 0;

	while index + 1 < bytes.len() {
		if bytes[index] == b'{' && bytes[index + 1] == b'{' {
			depth += 1;
			index += 2;
			continue;
		}
		if bytes[index] == b'}' && bytes[index + 1] == b'}' {
			if depth == 0 {
				return Ok(index);
			}
			depth -= 1;
			index += 2;
			continue;
		}
		index += 1;
	}

	Err(RuleError::Invalid(
		"unclosed placeholder in suggestion".to_string(),
	))
}

fn parse_part(input: &str) -> Result<ParsedPart, RuleError> {
	let input = input.trim();

	if input == "selection" {
		return Ok(Part::Selection.into());
	}

	if input == "command" {
		return Ok(Part::CommandAll.into());
	}

	if let Some(rest) = input.strip_prefix("command") {
		return Ok(Part::Command(parse_slice(rest)?).into());
	}

	if let Some(rest) = input.strip_prefix("typo") {
		let (slice, rest) = parse_indexed(rest)?;
		let choices = parse_choices(rest)?;
		if let Choices::Literal(values) = &choices
			&& values.len() == 1
		{
			return Err(RuleError::Invalid(
				"typo with one candidate must be `path`, `file`, or `{{shell}}`".to_string(),
			));
		}
		return Ok(Part::Typo { slice, choices }.into());
	}

	if let Some(rest) = input.strip_prefix("select") {
		let rest = rest.trim_start();
		let (slice, rest) = if rest.starts_with('[') {
			let (slice, rest) = parse_indexed(rest)?;
			(Some(slice), rest)
		} else {
			(None, rest)
		};
		if matches!(slice, Some(CommandSlice::Range { .. })) {
			return Err(RuleError::Invalid(
				"select index cannot be a range".to_string(),
			));
		}
		let choices = parse_choices(rest)?;
		return Ok(Part::Select { slice, choices }.into());
	}

	if let Some(regex) = input.strip_prefix("opt::") {
		return Ok(ParsedPart::Opt(regex.trim().to_string()));
	}

	if let Some(regex) = input.strip_prefix("cmd::") {
		return Ok(Part::CmdRegex(regex.trim().to_string()).into());
	}

	if let Some(regex) = input.strip_prefix("err::") {
		return Ok(Part::ErrRegex(regex.trim().to_string()).into());
	}

	if let Some(rest) = input.strip_prefix("shell") {
		let rest = rest.trim();
		let command = parens(rest)?;
		return Ok(Part::Shell(command.trim().to_string()).into());
	}

	Err(RuleError::Invalid(format!(
		"unknown placeholder: {{{{ {} }}}}",
		input
	)))
}

fn parse_slice(input: &str) -> Result<CommandSlice, RuleError> {
	let input = input.trim();
	let input = input.strip_prefix('[').unwrap_or(input);
	let input = input.strip_suffix(']').unwrap_or(input);

	if let Some((start, end)) = input.split_once(':') {
		let start = parse_index_part(start, 0)?;
		let end = if end.trim().is_empty() {
			None
		} else {
			Some(parse_index_part(end, 0)?)
		};
		return Ok(CommandSlice::Range { start, end });
	}

	Ok(CommandSlice::Single(parse_index_part(input, 0)?))
}

fn parse_index_part(input: &str, default: i32) -> Result<i32, RuleError> {
	let input = input.trim();
	if input.is_empty() {
		return Ok(default);
	}
	input
		.parse::<i32>()
		.map_err(|_| RuleError::Invalid(format!("invalid command index: {}", input)))
}

fn parse_indexed(input: &str) -> Result<(CommandSlice, &str), RuleError> {
	let rest = input.trim_start();
	let rest = rest
		.strip_prefix('[')
		.ok_or_else(|| RuleError::Invalid("missing index in placeholder".to_string()))?;
	let (index, rest) = rest
		.split_once(']')
		.ok_or_else(|| RuleError::Invalid("unclosed index in placeholder".to_string()))?;
	let slice = parse_slice(&format!("[{}]", index))?;
	Ok((slice, rest))
}

fn parse_choices(input: &str) -> Result<Choices, RuleError> {
	let input = input.trim();
	let input = parens(input)?;
	let input = input.trim();

	if input.starts_with("{{") && input.ends_with("}}") {
		let part = parse_part(&input[2..input.len() - 2])?;
		return match part {
			ParsedPart::Part(Part::Shell(command)) => Ok(Choices::Shell(command)),
			_ => Err(RuleError::Invalid(
				"only {{shell}} can be nested in a candidate list".to_string(),
			)),
		};
	}

	let values = input
		.split([',', '\n'])
		.map(str::trim)
		.filter(|value| !value.is_empty())
		.map(str::to_string)
		.collect::<Vec<String>>();

	if values.is_empty() {
		return Err(RuleError::Invalid(
			"candidate list cannot be empty".to_string(),
		));
	}

	if values.len() == 1 {
		match values[0].as_str() {
			"path" => return Ok(Choices::Path),
			"file" => return Ok(Choices::File),
			_ => {}
		}
	}

	Ok(Choices::Literal(values))
}

fn parens(input: &str) -> Result<&str, RuleError> {
	let input = input.trim();
	let input = input
		.strip_prefix('(')
		.and_then(|input| input.strip_suffix(')'))
		.ok_or_else(|| RuleError::Invalid("missing parentheses in placeholder".to_string()))?;
	Ok(input)
}
