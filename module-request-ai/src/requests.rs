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
	let mut json_buffer = String::new();
	let mut buffer = buffer::Buffer::new(config.ui.colors);

	while let Some(item) = stream.next().await {
		let item = item.unwrap();
		let str = std::str::from_utf8(&item).unwrap();

		json_buffer.push_str(str);

		if !json_buffer.contains("\n\ndata: ") {
			continue;
		}

		while json_buffer.contains("\n\ndata: ") {
			proc_stream_buffer(&mut json_buffer, &mut buffer);
		}
	}
	// remaining buffer
	json_buffer.push_str("\n\ndata: "); // hackey
	while json_buffer.contains("\n\ndata: ") {
		proc_stream_buffer(&mut json_buffer, &mut buffer);
	}

	let suggestions = buffer
		.buf
		.trim()
		.trim_end_matches("```")
		.trim()
		.trim_start_matches("<suggest>")
		.trim_end_matches("</suggest>")
		.replace("<br>", "<_PR_BR>");

	println!("{}", suggestions);
}

fn proc_stream_buffer(json_buffer: &mut String, buffer: &mut buffer::Buffer) {
	let data_loc = json_buffer.find("\n\ndata: ").unwrap();
	let binding = json_buffer.clone();
	let split = binding.split_at(data_loc);
	let json = split.0;
	*json_buffer = split.1.trim_start().to_string();

	proc_json(json, buffer);
}

fn proc_json(res: &str, buffer: &mut buffer::Buffer) {
	if let Some(data) = res.trim().strip_prefix("data: ") {
		if data == "[DONE]" {
			return;
		}
		let json = serde_json::from_str::<ChatCompletion>(data)
			.unwrap_or_else(|_| panic!("AI module: Failed to parse JSON content: {}", data));
		let choice = json.choices.first().expect("AI module: No choices found");
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
}
