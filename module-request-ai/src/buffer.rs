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

fn fill(str: &mut str) -> String {
	let width = termwidth();
	textwrap_fill(str, width)
}

fn clear_line() {
	let width = termwidth();
	let whitespace = " ".repeat(width);
	eprint!("\r{}\r", whitespace);
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
		self.buf.push_str(data);
		fix_data(&mut self.buf);
		self.buf = fill(&mut self.buf);

		if !self.buf.contains("\n") {
			eprint!("{}", data);
			std::io::stderr().flush().unwrap();
			return;
		}

		let slice = self.buf.split('\n').collect::<Vec<&str>>();

		for (idx, line) in slice.iter().enumerate() {
			clear_line();

			match line.trim() {
				"<note>" => {
					let warn = format!("{}:", t!("ai-suggestion"))
						.bold()
						.blue()
						.to_string();
					eprintln!("{}", warn);
				}
				"</note>" | "<suggest>" => {
					self.state = State::Buf;
				}
				"<think>" => {
					self.state = State::Think;
					let warn = format!("{}:", t!("ai-thinking")).bold().blue().to_string();
					eprintln!("{}", warn);
				}
				"```" => {}
				_ => {
					// just a new line
					if idx == slice.len() - 1 {
						eprint!("{}", line);
					} else {
						eprintln!("{}", line);
					}
				}
			}
			std::io::stderr().flush().unwrap();
		}
		self.buf = slice.last().unwrap().to_string();
	}

	fn proc_think(&mut self, data: &str) {
		self.buf.push_str(data);
		fix_data(&mut self.buf);
		self.buf = fill(&mut self.buf);

		if !self.buf.contains("\n") {
			eprint!("{}", data);
			std::io::stderr().flush().unwrap();
			return;
		}

		let slice = self.buf.split('\n').collect::<Vec<&str>>();

		for (idx, line) in slice.iter().enumerate() {
			clear_line();

			match line.trim() {
				"</think>" => {
					self.state = State::Write;
				}
				_ => {
					// just a new line
					if idx == slice.len() - 1 {
						eprint!("{}", line);
					} else {
						eprintln!("{}", line);
					}
				}
			}
			std::io::stderr().flush().unwrap();
		}
		self.buf = slice.last().unwrap().to_string();
	}
}

fn fix_data(data: &mut String) {
	let tag_list = ["<note>", "</note>", "<think>", "</think>", "```"];
	for tag in tag_list.iter() {
		if data.contains(tag) {
			let mut new_data = String::new();
			let mut remaining = data.as_str();
			while let Some(pos) = remaining.find(tag) {
				let split_before = &remaining[..pos].trim_end();
				let split_after = &remaining[pos + tag.len()..].trim_start();
				new_data.push_str(split_before);
				new_data.push('\n');
				new_data.push_str(tag);
				new_data.push('\n');

				remaining = split_after;
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
		let expected = "hello\n<note>\nfoo\n</note>\nbar".to_string();
		assert_eq!(data, expected);
	}
}
