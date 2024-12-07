use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

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
	pub command: String,
	pub note: String,
}

pub fn ai_suggestion(last_command: &str, error_msg: &str) -> Option<AISuggest> {
	if std::env::var("_PR_AI_DISABLE").is_ok() {
		return None;
	}

	let error_msg = if error_msg.len() > 300 {
		&error_msg[..300]
	} else {
		error_msg
	};

	let mut map = HashMap::new();
	map.insert("last_command", last_command);
	map.insert("error_msg", error_msg);

	let api_key = match std::env::var("_PR_AI_API_KEY") {
		Ok(key) => Some(key),
		Err(_) => {
			let env_key = option_env!("_DEF_PR_AI_API_KEY").map(|key| key.to_string());
			if env_key.is_none() {
				Some("Y29uZ3JhdHVsYXRpb25zLCB5b3UgZm91bmQgdGhlIHNlY3JldCE=".to_string())
			} else if env_key.as_ref().unwrap().is_empty() {
				None
			} else {
				env_key
			}
		}
	};

	let api_key = match api_key {
		Some(key) => {
			if key.is_empty() {
				return None;
			}
			key
		}
		None => {
			return None;
		}
	};

	let request_url = match std::env::var("_PR_AI_URL") {
		Ok(url) => url,
		Err(_) => "https://iff.envs.net/completions.py".to_string(),
	};
	let model = match std::env::var("_PR_AI_MODEL") {
		Ok(model) => model,
		Err(_) => "llama3-8b-8192".to_string(),
	};

	let user_locale = std::env::var("_PR_AI_LOCALE").unwrap_or("en-US".to_string());
	let set_locale = if !user_locale.starts_with("en") {
		format!(". Use language for locale {}", user_locale)
	} else {
		"".to_string()
	};

	let ai_prompt = format!(
		r#"
The command `{last_command}` returns the following error message: `{error_msg}`. Provide a command to fix it. Answer in the following JSON format without any extra text:
```
{{"command":"suggestion","note":"why it may fix the error{set_locale}"}}
```
"#
	);

	let res;
	let messages = Messages {
		messages: vec![Input {
			role: "user".to_string(),
			content: ai_prompt.to_string(),
		}],
		model,
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

		handle.url(&request_url).unwrap();
		handle.post(true).unwrap();
		handle.post_field_size(data.len() as u64).unwrap();

		let mut headers = List::new();
		headers
			.append(&format!("Authorization: Bearer {}", api_key))
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
			.arg(format!("Authorization: Bearer {}", api_key))
			.arg("-H")
			.arg("Content-Type: application/json")
			.arg("-d")
			.arg(serde_json::to_string(&messages).unwrap())
			.arg(request_url)
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
			eprintln!("Failed to parse JSON response: {}", res);
			return None;
		}
	};

	let content = &json["choices"][0]["message"]["content"];

	let suggestion: AISuggest = {
		let str = {
			let str = content.as_str();
			str?;
			str.expect("Failed to get content from response")
				.trim_start_matches("```")
				.trim_end_matches("```")
		};
		let json = serde_json::from_str(str);
		if json.is_err() {
			return None;
		}
		json.unwrap()
	};
	Some(suggestion)
}
