use std::io::Write;
use textwrap::fill as textwrap_fill;

fn termwidth() -> usize {
	use terminal_size::{terminal_size, Height, Width};
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
	pub buf: Vec<String>,
	state: State,
}

impl Buffer {
	pub fn new() -> Self {
		Buffer {
			buf: vec![],
			state: State::Write,
		}
	}
	pub fn proc(&mut self, data: &str) {
		match self.state {
			State::Write => self.proc_write(data),
			State::Think => self.proc_think(data),
			State::Buf => self.buf.push(data.to_string()),
		}
	}

	pub fn print_return_remain(&mut self) -> String {
		let buffered = self.buf.join("").trim().to_string();
		self.buf.clear();
		if self.state == State::Buf {
			return buffered;
		}

		let split = buffered.split_once("<suggestions>");
		if let Some((first, last)) = split {
			eprint!("{}", first);
			std::io::stdout().flush().unwrap();
			return last.to_string();
		}
		"".to_string()
	}

	fn proc_write(&mut self, data: &str) {
		if !data.contains("\n") {
			self.buf.push(data.to_string());
			let buffered = self.buf.join("").trim().to_string();
			let filled = fill(&buffered);
			if let Some(filled) = filled {
				self.buf.clear();
				let formatted = clear_format(&filled);
				eprint!("{}", formatted);
				self.buf
					.push(formatted.split_once("\n").unwrap().1.to_string());
				std::io::stdout().flush().unwrap();
				return;
			}
			eprint!("{}", data);
			std::io::stdout().flush().unwrap();
			return;
		}

		let mut data = data.to_string();
		while data.contains("\n") {
			let lines = data.split_once("\n").unwrap();
			let first = lines.0;
			let last = lines.1;
			self.buf.push(first.to_string());
			let buffered = self.buf.join("").trim().to_string();
			self.buf.clear();
			if buffered.ends_with("<note>") {
				let warn = format!("\r{}:", t!("ai-suggestion"))
					.bold()
					.blue()
					.to_string();
				let first = buffered.replace("<note>", &warn);
				eprintln!("{}", first);
				std::io::stdout().flush().unwrap();
			} else if buffered.ends_with("</note>") {
				let tag = "</note>";
				let whitespace = " ".repeat(tag.len());
				let formatted = format!("\r{}", whitespace);
				let first = buffered.replace("</note>", &formatted);
				eprintln!("{}", first);
				self.state = State::Buf;
				std::io::stdout().flush().unwrap();
			} else if buffered.ends_with("<think>") {
				let tag = "<think>";
				let warn = format!("\r{}:", t!("ai-thinking"))
					.bold()
					.blue()
					.to_string();
				let first = buffered.replace(tag, &warn);
				self.state = State::Think;
				eprintln!("{}", first);
				std::io::stdout().flush().unwrap();
			} else if buffered.ends_with("</think>") {
				let tag = "</think>";
				let whitespace = " ".repeat(tag.len());
				let formatted = format!("\r{}", whitespace);
				let first = buffered.replace(tag, &formatted);
				eprintln!("{}", first);
				std::io::stdout().flush().unwrap();
			} else if buffered.ends_with("```") {
				let tag = "```";
				let whitespace = " ".repeat(tag.len());
				let formatted = format!("\r{}", whitespace);
				let first = buffered.replace(tag, &formatted);
				eprintln!("{}", first);
				std::io::stdout().flush().unwrap();
			} else {
				eprintln!("{}", first);
				std::io::stdout().flush().unwrap();
			}
			data = last.to_string();
		}
		eprint!("{}", data);
	}

	fn proc_think(&mut self, data: &str) {
		if !data.contains("\n") {
			self.buf.push(data.to_string());
			let buffered = self.buf.join("").trim().to_string();
			let filled = fill(&buffered);
			if let Some(filled) = filled {
				self.buf.clear();
				let formatted = clear_format(&filled);
				eprint!("{}", formatted);
				self.buf
					.push(formatted.split_once("\n").unwrap().1.to_string());
				std::io::stdout().flush().unwrap();
				return;
			}
			eprint!("{}", data);
			std::io::stdout().flush().unwrap();
			return;
		}

		let mut data = data.to_string();
		while data.contains("\n") {
			let lines = data.split_once("\n").unwrap();
			let first = lines.0;
			let last = lines.1;
			self.buf.push(first.to_string());
			let buffered = self.buf.join("").trim().to_string();
			self.buf.clear();
			if buffered.ends_with("</think>") {
				let tag = "</think>";
				let whitespace = " ".repeat(tag.len());
				let formatted = format!("\r{}", whitespace);
				let first = buffered.replace(tag, &formatted);
				self.state = State::Write;
				eprintln!("{}", first);
				std::io::stdout().flush().unwrap();
			} else {
				eprintln!("{}", first);
				std::io::stdout().flush().unwrap();
			}
			data = last.to_string();
		}
		eprint!("{}", data);
	}
}
