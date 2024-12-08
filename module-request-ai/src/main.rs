use crate::requests::ai_suggestion;
use colored::Colorize;
use textwrap::{fill, termwidth};
mod requests;

#[macro_use]
extern crate rust_i18n;
i18n!("i18n", fallback = "en", minify_key = true);

fn main() -> Result<(), std::io::Error> {
	let command = std::env::var("_PR_LAST_COMMAND").expect("_PR_LAST_COMMAND not set");
	let error = std::env::var("_PR_ERROR_MSG").expect("_PR_ERROR_MSG not set");
	colored::control::set_override(true);

	#[cfg(debug_assertions)]
	{
		eprintln!("last_command: {}", command);
		eprintln!("error_msg: {}", error);
	}

	// skip for commands with no arguments,
	// very likely to be an error showing the usage
	if command.split_whitespace().count() == 1 {
		return Ok(());
	}
	let suggest = ai_suggestion(&command, &error);
	if let Some(suggest) = suggest {
		let warn = format!("{}:", t!("ai-suggestion")).bold().blue();
		let note = fill(&suggest.note, termwidth());

		eprintln!("{}\n{}", warn, note);
		let command = suggest.command;
		print!("{}<_PR_BR>", command);
	}
	Ok(())
}
