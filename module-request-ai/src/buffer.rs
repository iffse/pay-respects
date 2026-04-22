use std::io::Write;
use textwrap::fill as textwrap_fill;

fn termwidth() -> usize {
	use terminal_size::{Height, Width, terminal_size};
	let size = terminal_size();
	if let Some((Width(w), Height(_))) = size {
		std::cmp::min(w as usize, 80)
	} else {
		80
	}
}

fn fill(str: &str) -> Option<String> {
	let width = termwidth();
	let filled = textwrap_fill(str, width);
	if filled.contains('\n') {
		Some(filled)
	} else {
		None
	}
}

fn clear_format(str: &str) -> String {
	let width = termwidth();
	let whitespace = " ".repeat(width);
	let filled = textwrap_fill(str, width);
	format!("\r{}\r{}", whitespace, filled)
}

use colored::Colorize;

#[derive(PartialEq)]
enum State {
	Write,
	Think,
	Buf,
}

pub struct Buffer {
	pub buf: String,
	state: State,
}

impl Buffer {
	pub fn new() -> Self {
		Buffer {
			buf: String::new(),
			state: State::Write,
		}
	}
	pub fn proc(&mut self, data: &str) {
		match self.state {
			State::Write => self.proc_write(data),
			State::Think => self.proc_think(data),
			State::Buf => self.buf.push_str(data),
		}
	}

	fn proc_write(&mut self, data: &str) {
		let mut print = data.to_string();
		self.buf.push_str(data);
		fix_data(&mut self.buf);

		while self.buf.contains("\n") {
			let buf = self.buf.to_string();
			let lines = buf.split_once("\n").unwrap();

			let line = lines.0; // line before the newline
			self.buf = lines.1.to_string(); // remaining

			if line.ends_with("<note>") {
				let warn = format!("{}:", t!("ai-suggestion"))
					.bold()
					.blue()
					.to_string();
				print = format!("\r{}\n", warn);
			} else if line.ends_with("</note>") {
				self.state = State::Buf;
				let tag = "</note>";
				let clear = " ".repeat(tag.len()).to_string();
				print = format!("\r{}\n", clear);
			} else if line.ends_with("<think>") {
				self.state = State::Think;
				let warn = format!("{}:", t!("ai-thinking")).bold().blue().to_string();
				print = format!("\r{}\n", warn);
			} else if line.ends_with("```") {
				let tag = "```";
				let clear = " ".repeat(tag.len()).to_string();
				print = format!("\r{}\n", clear);
			}
		}
		eprint!("{}", print);
		std::io::stdout().flush().unwrap();
	}

	fn proc_think(&mut self, data: &str) {
		let mut print = data.to_string();
		self.buf.push_str(data);
		fix_data(&mut self.buf);

		while self.buf.contains("\n") {
			let buf = self.buf.to_string();
			let lines = buf.split_once("\n").unwrap();

			let line = lines.0; // line before the newline
			self.buf = lines.1.to_string(); // remaining

			if line.ends_with("</think>") {
				self.state = State::Write;
				let tag = "</think>";
				let clear = " ".repeat(tag.len());
				print = format!("\r{}\n", clear);
			}
		}

		eprint!("{}", print);
		std::io::stdout().flush().unwrap();
	}
}

fn fix_data(data: &mut String) {
	let tag_list = ["<note>", "</note>", "<think>", "</think>", "```"];
	for tag in tag_list.iter() {
		if data.contains(tag) {
			let mut new_data = String::new();
			let mut remaining = data.as_str();
			while let Some(pos) = remaining.find(tag) {
				let (head, tail) = remaining.split_at(pos + tag.len());
				new_data.push_str(head);
				if !tail.starts_with("\n") {
					new_data.push('\n');
				}
				remaining = tail;
			}
			new_data.push_str(remaining);
			*data = new_data;
		}
	}
}

#[allow(unused)]
mod tests {
	use super::*;

	#[test]
	fn test_fix_data() {
		let mut data = "hello<note>foo</note>bar".to_string();
		fix_data(&mut data);
		let expected = "hello<note>\nfoo</note>\nbar".to_string();
		assert_eq!(data, expected);
	}
}
