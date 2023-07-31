use std::path::Path;

use proc_macro::TokenStream;

#[proc_macro]
pub fn parse_rules(input: TokenStream) -> TokenStream {
	let directory = input.to_string().trim_matches('"').to_owned();
	let rules = get_rules(directory);
	let string_hashmap = gen_string_hashmap(rules);

	string_hashmap.parse().unwrap()
}

#[derive(serde::Deserialize)]
struct Rule {
	command: String,
	match_err: Vec<MatchError>,
}

#[derive(serde::Deserialize)]
struct MatchError {
	pattern: Vec<String>,
	suggest: Vec<String>,
}

fn get_rules(directory: String) -> Vec<Rule> {
	let files = std::fs::read_dir(directory).expect("Failed to read directory.");

	let mut rules = Vec::new();
	for file in files {
		let file = file.expect("Failed to read file.");
		let path = file.path();
		let path = path.to_str().expect("Failed to convert path to string.");

		let rule_file = parse_file(Path::new(path));
		rules.push(rule_file);
	}
	rules
}

fn gen_string_hashmap(rules: Vec<Rule>) -> String {
	let mut string_hashmap = String::from("HashMap::from([");
	for rule in rules {
		let command = rule.command.to_owned();
		string_hashmap.push_str(&format!("(\"{}\", vec![", command));
		for match_err in rule.match_err {
			let pattern = match_err
				.pattern
				.iter()
				.map(|x| x.to_lowercase())
				.collect::<Vec<String>>();
			let suggest = match_err.
				suggest
				.iter()
				.map(|x| x.to_lowercase())
				.collect::<Vec<String>>();
			string_hashmap.push_str(&format!(
				"(vec![\"{}\"], vec![\"{}\"]),",
				pattern.join("\", \""),
				suggest.join("\", \"")
			));
		}
		string_hashmap.push_str("]),");
	}
	string_hashmap.push_str("])");
	string_hashmap
}

fn parse_file(file: &Path) -> Rule {
	let file = std::fs::read_to_string(file).expect("Failed to read file.");
	toml::from_str(&file).expect("Failed to parse toml.")
}
