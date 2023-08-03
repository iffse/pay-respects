use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

fn rtag (name: &str, x: i32, y: String) -> TokenStream2 {
	let tag = format!("{}{} = {}", name, x, y);
	let tag: TokenStream2 = tag.parse().unwrap();
	tag
}

fn tag (name: &str, x: i32) -> String {
	let tag = format!("{{{}{}}}", name, x);
	let tag = tag.as_str();
	let tag = tag.to_owned();
	tag
}

pub fn opts(suggest: &mut String, replace_list: &mut Vec<TokenStream2>, opt_list: &mut Vec<TokenStream2>) {
	let mut replace_tag = 0;
	let tag_name = "opts";
	while suggest.contains("{{opt::") {
		let placeholder_start = "{{opt::";
		let placeholder_end = "}}";

		let start_index = suggest.find(placeholder_start).unwrap();
		let end_index = suggest[start_index..].find(placeholder_end).unwrap()
			+ start_index
			+ placeholder_end.len();

		let placeholder = start_index..end_index;

		let args = start_index + placeholder_start.len()..end_index - placeholder_end.len();
		let opt = &suggest[args.to_owned()];
		let regex = opt.trim();
		let current_tag = tag(tag_name, replace_tag);
		let token_tag: TokenStream2 = format!("{}{}", tag_name, replace_tag).parse().unwrap();
		let command = quote! {
			let #token_tag = opt_regex(#regex, &mut last_command);
		};
		opt_list.push(command);

		replace_list.push(rtag(tag_name, replace_tag, current_tag.to_owned()));
		suggest.replace_range(placeholder, &current_tag);
		replace_tag += 1;
	}
}

pub fn command(suggest: &mut String, replace_list: &mut Vec<TokenStream2>) {
	let mut replace_tag = 0;
	let tag_name = "command";
	while suggest.contains("{{command") {
		let placeholder_start = "{{command";
		let placeholder_end = "}}";

		let start_index = suggest.find(placeholder_start).unwrap();
		let end_index = suggest[start_index..].find(placeholder_end).unwrap()
			+ start_index
			+ placeholder_end.len();

		let placeholder = start_index..end_index;

		let args = start_index + placeholder_start.len()..end_index - placeholder_end.len();
		let range = suggest[args.to_owned()].trim_matches(|c| c == '[' || c == ']');
		if let Some((start, end)) = range.split_once(':') {
			let mut start_string = start.to_string();
			let start = start.parse::<i32>().unwrap_or(0);
			if start < 0 {
				start_string = format!("split_command.len() {}", start);
			};
			let end_string;
			let parsed_end = end.parse::<i32>();
			if parsed_end.is_err() {
				end_string = String::from("split_command.len()");
			} else {
				let end = parsed_end.clone().unwrap();
				if end < 0 {
					end_string = format!("split_command.len() {}", end + 1);
				} else {
					end_string = (end + 1).to_string();
				}
			};

			let command = format!{r#"split_command[{}..{}].join(" ")"#, start_string, end_string};

			replace_list.push(rtag(tag_name, replace_tag, command));
			suggest.replace_range(placeholder, &tag(tag_name, replace_tag));
			replace_tag += 1;
		} else {
			let range = range.parse::<i32>().unwrap_or(0);
			let command = format!("split_command[{}]", range);

			replace_list.push(rtag(tag_name, replace_tag, command));
			suggest.replace_range(placeholder, &tag(tag_name, replace_tag));
			replace_tag += 1;
		}
	}
}

pub fn typo(suggest: &mut String, replace_list: &mut Vec<TokenStream2>) {
	let mut replace_tag = 0;
	let tag_name = "typo";

	while suggest.contains("{{typo") {
		let placeholder_start = "{{typo";
		let placeholder_end = "}}";

		let start_index = suggest.find(placeholder_start).unwrap();
		let end_index = suggest[start_index..].find(placeholder_end).unwrap()
			+ start_index
			+ placeholder_end.len();

		let placeholder = start_index..end_index;
		let args = start_index + placeholder_start.len()..end_index - placeholder_end.len();

		let string_index;
		if suggest.contains('[') {
			let split = suggest[args.to_owned()]
				.split(&['[', ']'])
				.collect::<Vec<&str>>();
			let command_index = split[1].parse::<i32>().unwrap();
			if command_index < 0 {
				// command_index += split_command.len() as i32;
				string_index = format!("split_command.len() {}", command_index);
			} else {
				string_index = command_index.to_string();
			}
		} else {
			unreachable!("Typo suggestion must have a command index");
		}
		let mut match_list = Vec::new();
		if suggest.contains('(') {
			let split = suggest[args.to_owned()]
				.split(&['(', ')'])
				.collect::<Vec<&str>>();
			match_list = split[1].trim().split(',').collect::<Vec<&str>>();
		}

		let match_list = match_list
			.iter()
			.map(|s| s.trim().to_string())
			.collect::<Vec<String>>();
		let string_match_list = match_list.join(r#"".to_string(), ""#);
		let string_match_list = format!(r#""{}".to_string()"#, string_match_list);

		let command = format!("suggest_typo(&split_command[{}], vec![{}])", string_index, string_match_list);

		replace_list.push(rtag(tag_name, replace_tag, command));
		suggest.replace_range(placeholder, &tag(tag_name, replace_tag));
		replace_tag += 1;
	}

}
