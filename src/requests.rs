use std::collections::HashMap;

use reqwest::blocking::Client;
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
			// I am keeping the key so anyone can use it out of the box
			// Please, don't abuse the key and try to use your own key
			if env_key.is_none() {
				Some("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".to_string())
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
		Err(_) => "https://api.groq.com/openai/v1/chat/completions".to_string(),
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

	let messages = Messages {
		messages: vec![Input {
			role: "user".to_string(),
			content: ai_prompt.to_string(),
		}],
		model,
	};

	let client = Client::new();
	let res = client
		.post(&request_url)
		.header("Authorization", format!("Bearer {}", api_key))
		.header("Content-Type", "application/json")
		.json(&messages)
		.timeout(std::time::Duration::from_secs(10))
		.send();

	let res = match res {
		Ok(res) => res,
		Err(_) => {
			return None;
		}
	};

	let res = &res.text().unwrap();
	let json: Value = {
		let json = serde_json::from_str(res);
		if json.is_err() {
			return None;
		}
		json.unwrap()
	};

	let content = &json["choices"][0]["message"]["content"];

	let suggestion: AISuggest = {
		let str = {
			let str = content.as_str();
			str?;
			str.unwrap()
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
