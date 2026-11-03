use askama::Template;
use serde_json::Value;

use futures_util::StreamExt;
use serde::Deserialize;

use pay_respects_utils::strings::print_error;

use crate::buffer;
use crate::config::Config;

struct Conf {
	key: String,
	url: String,
	model: String,
}

// #[derive(Serialize)]
// struct Input {
// 	role: String,
// 	content: String,
// }
//
// #[derive(Serialize)]
// struct Messages {
// 	messages: Vec<Input>,
// 	model: String,
// 	stream: bool,
// 	extra_body: Option<Value>,
// }

#[derive(Debug, Default, Deserialize)]
pub struct ChatCompletion {
	// id: String,
	// object: String,
	// created: usize,
	#[serde(default)]
	choices: Vec<Choice>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Choice {
	#[serde(default)]
	delta: Delta,
	// index: usize,
	// finish_reason: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Delta {
	content: Option<String>,
}

#[derive(Template)]
#[template(path = "prompt.txt")]
struct AiPrompt<'a> {
	last_command: &'a str,
	error_msg: &'a str,
	additional_prompt: &'a str,
	set_locale: &'a str,
}

#[derive(Template)]
#[template(path = "request.txt")]
struct Request {
	extra: Option<String>,
	extra_body: Option<String>,
}

pub async fn ai_suggestion(last_command: &str, error_msg: &str, locale: &str, config: &Config) {
	let conf = match Conf::new(config) {
		Some(conf) => conf,
		None => {
			return;
		}
	};

	let char_limit = 500;

	let error_msg = if error_msg.chars().count() > 500 + 3 {
		let half = char_limit / 2;
		format!(
			"{}...{}",
			error_msg.chars().take(half).collect::<String>(),
			error_msg
				.chars()
				.rev()
				.take(half)
				.collect::<String>()
				.chars()
				.rev()
				.collect::<String>()
		)
	} else {
		error_msg.to_string()
	};

	let user_locale = {
		let locale = std::env::var("_PR_AI_LOCALE")
			.ok()
			.or_else(|| config.locale.clone())
			.unwrap_or_else(|| locale.to_string());
		if locale.len() < 2 {
			"en-US".to_string()
		} else {
			locale
		}
	};

	let set_locale = if !user_locale.starts_with("en") {
		format!(". Use language for locale {}", user_locale)
	} else {
		"".to_string()
	};

	let addtional_prompt = std::env::var("_PR_AI_ADDITIONAL_PROMPT")
		.ok()
		.or_else(|| config.additional_prompt.clone())
		.unwrap_or_default();

	let ai_prompt = AiPrompt {
		last_command,
		error_msg: &error_msg,
		additional_prompt: &addtional_prompt,
		set_locale: &set_locale,
	}
	.render()
	.unwrap()
	.trim()
	.to_string();

	pay_respects_utils::dlog!(6, "AI module: AI prompt: {}", ai_prompt);

	let extra = std::env::var("_PR_AI_EXTRA")
		.ok()
		.or_else(|| config.extra.clone());
	let extra_body = std::env::var("_PR_AI_EXTRA_BODY")
		.ok()
		.or_else(|| config.extra_body.clone());

	let body = Request { extra, extra_body };
	let body = body.render().unwrap();
	let mut json_body: Value = serde_json::from_str(&body).unwrap();
	json_body["model"] = Value::String(conf.model);
	json_body["messages"][0]["content"] = Value::String(ai_prompt);
	json_body["stream"] = Value::Bool(true);

	let client = reqwest::Client::new();
	let res = client
		.post(&conf.url)
		.body(serde_json::to_string(&json_body).unwrap())
		.header("Content-Type", "application/json")
		.bearer_auth(&conf.key)
		.send()
		.await
		.unwrap();

	if res.status() != 200 {
		print_error(&format!("AI request failed with status: {}", res.status()));
		print_error(&format!(
			"AI error message:\n  {}",
			res.text().await.unwrap().replace("\n", "\n  ")
		));
		return;
	}

	let mut stream = res.bytes_stream();
	let mut buffer = buffer::Buffer::new(config.ui.colors);
	let mut pending = Vec::new();

	while let Some(item) = stream.next().await {
		match item {
			Ok(item) => proc_chunk(&mut pending, &item, &mut buffer),
			Err(err) => {
				print_error(&format!("AI module: stream error: {}", err));
				break;
			}
		}
	}

	// Process a trailing event that was not followed by a blank line.
	pending.extend_from_slice(b"\n\n");
	while let Some(event) = next_event(&mut pending) {
		proc_event(&event, &mut buffer);
	}

	let suggestions = buffer
		.buf
		.trim()
		.trim_end_matches("```")
		.trim()
		.trim_start_matches("<suggest>")
		.trim_end_matches("</suggest>")
		.replace("<br>", "<_PR_BR>");

	if suggestions.is_empty() {
		return;
	}

	println!("{}", suggestions);
}

/// Feeds a chunk of the response to the event parser. Carriage returns are
/// dropped since they are only used by the server as line separators.
fn proc_chunk(pending: &mut Vec<u8>, chunk: &[u8], buffer: &mut buffer::Buffer) {
	pending.extend(chunk.iter().copied().filter(|&byte| byte != b'\r'));

	while let Some(event) = next_event(pending) {
		proc_event(&event, buffer);
	}
}

/// Splits the next complete server-sent event off the buffer.
fn next_event(pending: &mut Vec<u8>) -> Option<Vec<u8>> {
	let end = pending.windows(2).position(|window| window == b"\n\n")?;
	let mut event = pending.drain(..end + 2).collect::<Vec<u8>>();
	event.truncate(end);
	Some(event)
}

fn proc_event(event: &[u8], buffer: &mut buffer::Buffer) {
	let event = String::from_utf8_lossy(event);

	for line in event.lines() {
		let Some(data) = line.trim().strip_prefix("data:") else {
			continue;
		};
		let data = data.trim();
		if data.is_empty() || data == "[DONE]" {
			continue;
		}
		proc_json(data, buffer);
	}
}

fn proc_json(data: &str, buffer: &mut buffer::Buffer) {
	let json = match serde_json::from_str::<ChatCompletion>(data) {
		Ok(json) => json,
		Err(err) => {
			pay_respects_utils::dlog!(3, "AI module: skipping stream chunk: {} ({})", data, err);
			return;
		}
	};

	// Some providers send chunks with an empty choice list before or after the
	// ones carrying content.
	for choice in &json.choices {
		if let Some(content) = &choice.delta.content {
			buffer.proc(content);
		}
	}
}

impl Conf {
	pub fn new(config: &Config) -> Option<Self> {
		let key = match std::env::var("_PR_AI_API_KEY") {
			Ok(key) => key,
			Err(_) => match &config.api_key {
				Some(key) => key.clone(),
				None => option_env!("_DEF_PR_AI_API_KEY")
					.unwrap_or("Y29uZ3JhdHVsYXRpb25zLCB5b3UgZm91bmQgdGhlIHNlY3JldCE=")
					.to_string(),
			},
		};
		if key.is_empty() {
			return None;
		}

		let url = match std::env::var("_PR_AI_URL") {
			Ok(url) => url,
			Err(_) => match &config.api_url {
				Some(url) => url.clone(),
				None => option_env!("_DEF_PR_AI_URL")
					.unwrap_or("https://pay-respects-serverless.iffse.eu.org/")
					.to_string(),
			},
		};
		if url.is_empty() {
			return None;
		}

		let model = match std::env::var("_PR_AI_MODEL") {
			Ok(model) => model,
			Err(_) => match &config.model {
				Some(model) => model.clone(),
				None => option_env!("_DEF_PR_AI_MODEL").unwrap_or("uwu").to_string(),
			},
		};
		if model.is_empty() {
			return None;
		}

		Some(Conf { key, url, model })
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn dynamic_values_are_at_the_end() {
		let prompt = AiPrompt {
			last_command: "ZZ_LAST_COMMAND_ZZ",
			error_msg: "ZZ_ERROR_MSG_ZZ",
			additional_prompt: "",
			set_locale: "",
		}
		.render()
		.unwrap();

		let instructions = prompt
			.find("Your answer should be in the following format")
			.unwrap();
		let command = prompt.find("ZZ_LAST_COMMAND_ZZ").unwrap();
		let error = prompt.find("ZZ_ERROR_MSG_ZZ").unwrap();

		assert!(instructions < command);
		assert!(command < error);
	}

	fn process(chunks: &[&[u8]]) -> String {
		let mut buffer = buffer::Buffer::new(pay_respects_utils::ui::Colors::default());
		let mut pending = Vec::new();
		for chunk in chunks {
			proc_chunk(&mut pending, chunk, &mut buffer);
		}
		pending.extend_from_slice(b"\n\n");
		while let Some(event) = next_event(&mut pending) {
			proc_event(&event, &mut buffer);
		}
		buffer.buf
	}

	#[test]
	fn skips_chunks_with_empty_choices() {
		let stream = concat!(
			"data: {\"choices\":[]}\n\n",
			"data: {\"choices\":[{\"delta\":{\"content\":\"gi\"}}]}\n\n",
			"data: {\"choices\":[],\"usage\":{\"total_tokens\":1}}\n\n",
			"data: {\"choices\":[{\"delta\":{\"content\":\"t status\"}}]}\n\n",
			"data: [DONE]\n\n",
		);

		assert_eq!(process(&[stream.as_bytes()]), "git status");
	}

	#[test]
	fn skips_malformed_chunks() {
		let stream = concat!(
			"data: not json\n\n",
			"data: {\"choices\":[{\"delta\":{\"content\":\"ok\"}}]}\n\n",
		);

		assert_eq!(process(&[stream.as_bytes()]), "ok");
	}

	#[test]
	fn tolerates_crlf_and_chunks_without_delta() {
		let stream = concat!(
			"data: {\"choices\":[{\"finish_reason\":\"stop\"}]}\r\n\r\n",
			"data: {\"choices\":[{\"delta\":{\"content\":\"a\"}}]}\r\n\r\n",
			"data: {\"choices\":[{\"delta\":{\"content\":\"b\"}}]}\r\n\r\n",
		);

		assert_eq!(process(&[stream.as_bytes()]), "ab");
	}

	#[test]
	fn tolerates_split_utf8() {
		let stream = "data: {\"choices\":[{\"delta\":{\"content\":\"你好\"}}]}\n\n";
		let bytes = stream.as_bytes();
		let split = bytes.iter().position(|&byte| byte == 0xE4).unwrap() + 1;
		let (first, second) = bytes.split_at(split);

		assert_eq!(process(&[first, second]), "你好");
	}
}
