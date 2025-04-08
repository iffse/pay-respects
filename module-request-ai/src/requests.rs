use askama::Template;
use std::collections::HashMap;

use sys_locale::get_locale;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::buffer;

struct Conf {
	key: String,
	url: String,
	model: String,
}

#[derive(Serialize)]
struct Input {
	role: String,
	content: String,
}

#[derive(Serialize)]
struct Messages {
	messages: Vec<Input>,
	model: String,
	stream: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletion {
	// id: String,
	// object: String,
	// created: usize,
	choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
	delta: Delta,
	// index: usize,
	// finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
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

pub async fn ai_suggestion(last_command: &str, error_msg: &str) {
	if std::env::var("_PR_AI_DISABLE").is_ok() {
		return;
	}

	let conf = match Conf::new() {
		Some(conf) => conf,
		None => {
			return;
		}
	};

	let error_msg = if error_msg.len() > 300 {
		&error_msg[..300]
	} else {
		error_msg
	};

	let mut map = HashMap::new();
	map.insert("last_command", last_command);
	map.insert("error_msg", error_msg);

	let user_locale = {
		let locale = std::env::var("_PR_AI_LOCALE")
			.unwrap_or_else(|_| get_locale().unwrap_or("en-us".to_string()));
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

	let addtional_prompt = if std::env::var("_PR_AI_ADDITIONAL_PROMPT").is_ok() {
		std::env::var("_PR_AI_ADDITIONAL_PROMPT").unwrap()
	} else {
		"".to_string()
	};

	let ai_prompt = AiPrompt {
		last_command,
		error_msg,
		additional_prompt: &addtional_prompt,
		set_locale: &set_locale,
	}
	.render()
	.unwrap()
	.trim()
	.to_string();

	#[cfg(debug_assertions)]
	eprintln!("AI module: AI prompt: {}", ai_prompt);

	// let res;
	let body = Messages {
		messages: vec![Input {
			role: "user".to_string(),
			content: ai_prompt.trim().to_string(),
		}],
		model: conf.model,
		stream: true,
	};

	let client = reqwest::Client::new();
	let res = client
		.post(&conf.url)
		.body(serde_json::to_string(&body).unwrap())
		.header("Content-Type", "application/json")
		.bearer_auth(&conf.key)
		.send()
		.await;

	let mut stream = res.unwrap().bytes_stream();
	let mut json_buffer = vec![];
	let mut buffer = buffer::Buffer::new();
	while let Some(item) = stream.next().await {
		let item = item.unwrap();
		let str = std::str::from_utf8(&item).unwrap();

		if json_buffer.is_empty() {
			json_buffer.push(str.to_string());
			continue;
		}

		if !str.contains("\n\ndata: {") {
			json_buffer.push(str.to_string());
			continue;
		}
		let data_loc = str.find("\n\ndata: {").unwrap();
		let split = str.split_at(data_loc);
		json_buffer.push(split.0.to_string());
		let working_str = json_buffer.join("");
		json_buffer.clear();
		json_buffer.push(split.1.to_string());

		for part in working_str.split("\n\n") {
			if let Some(data) = part.strip_prefix("data: ") {
				if data == "[DONE]" {
					break;
				}
				let json = serde_json::from_str::<ChatCompletion>(data).unwrap_or_else(|_| {
					panic!("AI module: Failed to parse JSON content: {}", data)
				});
				let choice = json.choices.first().expect("AI module: No choices found");
				if let Some(content) = &choice.delta.content {
					buffer.proc(content);
				}
			}
		}
	}
	if !json_buffer.is_empty() {
		let working_str = json_buffer.join("");
		for part in working_str.split("\n\n") {
			if let Some(data) = part.strip_prefix("data: ") {
				if data == "[DONE]" {
					break;
				}
				let json = serde_json::from_str::<ChatCompletion>(data).unwrap_or_else(|_| {
					panic!("AI module: Failed to parse JSON content: {}", data)
				});
				let choice = json.choices.first().expect("AI module: No choices found");
				if let Some(content) = &choice.delta.content {
					buffer.proc(content);
				}
			}
		}
		json_buffer.clear();
	}
	let suggestions = buffer
		.print_return_remain()
		.trim()
		.trim_end_matches("```")
		.trim()
		.trim_start_matches("<suggestions>")
		.trim_end_matches("</suggestions>")
		.replace("<br>", "<_PR_BR>");

	println!("{}", suggestions);
}

impl Conf {
	pub fn new() -> Option<Self> {
		let key = match std::env::var("_PR_AI_API_KEY") {
			Ok(key) => key,
			Err(_) => {
				if let Some(key) = option_env!("_DEF_PR_AI_API_KEY") {
					key.to_string()
				} else {
					"Y29uZ3JhdHVsYXRpb25zLCB5b3UgZm91bmQgdGhlIHNlY3JldCE=".to_string()
				}
			}
		};
		if key.is_empty() {
			return None;
		}

		let url = match std::env::var("_PR_AI_URL") {
			Ok(url) => url,
			Err(_) => {
				if let Some(url) = option_env!("_DEF_PR_AI_URL") {
					url.to_string()
				} else {
					"https://iff.envs.net/stream-completions.py".to_string()
				}
			}
		};
		if url.is_empty() {
			return None;
		}

		let model = match std::env::var("_PR_AI_MODEL") {
			Ok(model) => model,
			Err(_) => {
				if let Some(model) = option_env!("_DEF_PR_AI_MODEL") {
					model.to_string()
				} else {
					"{{ _PR_AI_MODEL }}".to_string()
				}
			}
		};
		if model.is_empty() {
			return None;
		}

		Some(Conf { key, url, model })
	}
}
