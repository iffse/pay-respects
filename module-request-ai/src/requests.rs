use std::collections::HashMap;
use sys_locale::get_locale;

use serde::{Deserialize, Serialize};
use serde_json::Value;

struct Conf {
	key: String,
	url: String,
	model: String,
}

#[derive(Serialize, Deserialize)]
struct Input {
	role: String,
	content: String,
}

#[derive(Serialize, Deserialize)]
struct Messages {
	messages: Vec<Input>,
	model: String,
}

#[derive(Serialize, Deserialize)]
pub struct AISuggest {
	pub commands: Vec<String>,
	pub note: String,
}

pub fn ai_suggestion(last_command: &str, error_msg: &str) -> Option<AISuggest> {
	if std::env::var("_PR_AI_DISABLE").is_ok() {
		return None;
	}

	let conf = match Conf::new() {
		Some(conf) => conf,
		None => {
			return None;
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

	let ai_prompt = format!(
		r#"
`{last_command}` returns the following error message: `{error_msg}`. Provide possible commands to fix it. Answer in the following exact JSON template without any extra text:
```
{{"commands":["command 1","command 2"],"note":"why they may fix the error{set_locale}"}}
```
"#
	);

	let res;
	let messages = Messages {
		messages: vec![Input {
			role: "user".to_string(),
			content: ai_prompt.trim().to_string(),
		}],
		model: conf.model,
	};

	#[cfg(feature = "libcurl")]
	{
		use curl::easy::Easy as Curl;
		use curl::easy::List;
		use std::io::Read;

		let str_json = serde_json::to_string(&messages).unwrap();
		let mut data = str_json.as_bytes();

		let mut dst = Vec::new();
		let mut handle = Curl::new();

		handle.url(&conf.url).unwrap();
		handle.post(true).unwrap();
		handle.post_field_size(data.len() as u64).unwrap();

		let mut headers = List::new();
		headers
			.append(&format!("Authorization: Bearer {}", conf.key))
			.unwrap();
		headers.append("Content-Type: application/json").unwrap();
		handle.http_headers(headers).unwrap();

		{
			let mut transfer = handle.transfer();

			transfer
				.read_function(|buf| Ok(data.read(buf).unwrap_or(0)))
				.unwrap();

			transfer
				.write_function(|buf| {
					dst.extend_from_slice(buf);
					Ok(buf.len())
				})
				.unwrap();

			transfer.perform().expect("Failed to perform request");
		}

		res = String::from_utf8(dst).unwrap();
	}
	#[cfg(not(feature = "libcurl"))]
	{
		let proc = std::process::Command::new("curl")
			.arg("-X")
			.arg("POST")
			.arg("-H")
			.arg(format!("Authorization: Bearer {}", conf.key))
			.arg("-H")
			.arg("Content-Type: application/json")
			.arg("-d")
			.arg(serde_json::to_string(&messages).unwrap())
			.arg(conf.url)
			.output();

		let out = match proc {
			Ok(proc) => proc.stdout,
			Err(_) => {
				return None;
			}
		};
		res = String::from_utf8(out).unwrap();
	}
	let json: Value = {
		let json = serde_json::from_str(&res);
		if let Ok(json) = json {
			json
		} else {
			eprintln!("AI module: Failed to parse JSON response: {}", res);
			return None;
		}
	};

	let content = &json["choices"][0]["message"]["content"];

	let suggestion: AISuggest = {
		let str = {
			let str = content.as_str();
			str?;
			str.expect("AI module: Failed to get content from response")
				.trim_start_matches("```")
				.trim_start_matches("json")
				.trim_end_matches("```")
		};
		let json = serde_json::from_str(str);
		if json.is_err() {
			eprintln!("AI module: Failed to parse JSON content: {}", str);
			return None;
		}
		json.unwrap()
	};
	Some(suggestion)
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
					"https://iff.envs.net/completions.py".to_string()
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
					"llama3-70b-8192".to_string()
				}
			}
		};
		if model.is_empty() {
			return None;
		}

		Some(Conf { key, url, model })
	}
}
