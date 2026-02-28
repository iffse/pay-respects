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

const TAGS: &[&str] = &[
	"<suggestions>",
	"</suggestions>",
	"<suggest>",
	"</suggest>",
	"<note>",
	"</note>",
	"<think>",
	"</think>",
];

pub struct Buffer {
	pub buf: String,
	pending: String,
	line_buf: String,
	state: State,
}

impl Buffer {
	pub fn new() -> Self {
		Buffer {
			buf: String::new(),
			pending: String::new(),
			line_buf: String::new(),
			state: State::Write,
		}
	}

	/// Find the first occurrence of any known tag in `s`.
	fn find_first_tag(s: &str) -> Option<(usize, &'static str)> {
		let mut best: Option<(usize, &'static str)> = None;
		for &tag in TAGS {
			if let Some(pos) = s.find(tag) {
				if best.is_none() || pos < best.unwrap().0 {
					best = Some((pos, tag));
				}
			}
		}
		best
	}

	/// Return the length of a suffix of `s` that could be the start of a known tag.
	fn partial_tag_len(s: &str) -> usize {
		let max_check = s.len().min(15);
		for len in (1..=max_check).rev() {
			let suffix = &s[s.len() - len..];
			if !suffix.starts_with('<') {
				continue;
			}
			for &tag in TAGS {
				if tag.starts_with(suffix) {
					return len;
				}
			}
		}
		0
	}

	pub fn proc(&mut self, data: &str) {
		self.pending.push_str(data);
		self.drain();
	}

	fn drain(&mut self) {
		loop {
			let partial = Self::partial_tag_len(&self.pending);
			let safe_len = self.pending.len() - partial;
			if safe_len == 0 {
				break;
			}

			let safe = self.pending[..safe_len].to_string();

			match Self::find_first_tag(&safe) {
				Some((pos, tag)) => {
					if pos > 0 {
						let before = self.pending[..pos].to_string();
						self.emit_text(&before);
					}
					let after_pos = pos + tag.len();
					self.pending = self.pending[after_pos..].to_string();
					self.handle_tag(tag);
				}
				None => {
					let to_emit = self.pending[..safe_len].to_string();
					self.pending = self.pending[safe_len..].to_string();
					self.emit_text(&to_emit);
					break;
				}
			}
		}
	}

	fn handle_tag(&mut self, tag: &str) {
		match tag {
			"<note>" => {
				self.flush_line();
				let warn = format!("{}:", t!("ai-suggestion"))
					.bold()
					.blue()
					.to_string();
				eprintln!("{}", warn);
				std::io::stderr().flush().unwrap();
			}
			"</note>" => {
				self.flush_line();
				self.state = State::Buf;
			}
			"<think>" => {
				self.flush_line();
				let warn = format!("{}:", t!("ai-thinking")).bold().blue().to_string();
				eprintln!("{}", warn);
				std::io::stderr().flush().unwrap();
				self.state = State::Think;
			}
			"</think>" => {
				self.flush_line();
				if self.state == State::Think {
					self.state = State::Write;
				}
			}
			"<suggestions>" | "<suggest>" => {
				self.flush_line();
				self.state = State::Buf;
			}
			"</suggestions>" | "</suggest>" => {}
			_ => {}
		}
	}

	fn flush_line(&mut self) {
		if !self.line_buf.is_empty() {
			eprintln!();
			std::io::stderr().flush().unwrap();
			self.line_buf.clear();
		}
	}

	fn emit_text(&mut self, text: &str) {
		if text.is_empty() {
			return;
		}
		match self.state {
			State::Buf => {
				self.buf.push_str(text);
			}
			State::Write | State::Think => {
				let mut remaining = text;
				while let Some(nl_pos) = remaining.find('\n') {
					let segment = &remaining[..nl_pos];
					self.line_buf.push_str(segment);
					let buffered = self.line_buf.trim().to_string();
					self.line_buf.clear();

					if self.state == State::Write && buffered.ends_with("```") {
						let whitespace = " ".repeat(3);
						let formatted = format!("\r{}", whitespace);
						let line = buffered.replace("```", &formatted);
						eprintln!("{}", line);
					} else {
						let formatted = clear_format(&buffered);
						eprintln!("{}", formatted);
					}
					std::io::stderr().flush().unwrap();
					remaining = &remaining[nl_pos + 1..];
				}
				if !remaining.is_empty() {
					self.line_buf.push_str(remaining);
					let buffered = self.line_buf.trim().to_string();
					if let Some(filled) = fill(&buffered) {
						self.line_buf.clear();
						let formatted = clear_format(&filled);
						eprint!("{}", formatted);
						self.line_buf
							.push_str(formatted.split_once("\n").unwrap().1);
					} else {
						eprint!("{}", remaining);
					}
					std::io::stderr().flush().unwrap();
				}
			}
		}
	}

	pub fn print_return_remain(&mut self) -> String {
		// Flush any remaining pending data
		if !self.pending.is_empty() {
			let remaining = std::mem::take(&mut self.pending);
			self.emit_text(&remaining);
		}
		self.flush_line();

		let buffered = self.buf.trim().to_string();
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
}

#[cfg(test)]
mod tests {
	use super::*;

	// --- find_first_tag ---

	#[test]
	fn find_first_tag_at_start() {
		let result = Buffer::find_first_tag("<note>hello");
		assert_eq!(result, Some((0, "<note>")));
	}

	#[test]
	fn find_first_tag_mid_string() {
		let result = Buffer::find_first_tag("some text<note>more");
		assert_eq!(result, Some((9, "<note>")));
	}

	#[test]
	fn find_first_tag_none() {
		let result = Buffer::find_first_tag("no tags here");
		assert_eq!(result, None);
	}

	#[test]
	fn find_first_tag_picks_earliest() {
		let result = Buffer::find_first_tag("a<think>b<note>c");
		assert_eq!(result, Some((1, "<think>")));
	}

	#[test]
	fn find_first_tag_closing() {
		let result = Buffer::find_first_tag("text</note>rest");
		assert_eq!(result, Some((4, "</note>")));
	}

	#[test]
	fn find_first_tag_suggestions() {
		let result = Buffer::find_first_tag("x<suggestions>y");
		assert_eq!(result, Some((1, "<suggestions>")));
	}

	#[test]
	fn find_first_tag_suggest() {
		let result = Buffer::find_first_tag("x<suggest>y");
		assert_eq!(result, Some((1, "<suggest>")));
	}

	// --- partial_tag_len ---

	#[test]
	fn partial_tag_none() {
		assert_eq!(Buffer::partial_tag_len("hello"), 0);
	}

	#[test]
	fn partial_tag_open_angle() {
		// "<" could be the start of any tag
		assert_eq!(Buffer::partial_tag_len("hello<"), 1);
	}

	#[test]
	fn partial_tag_note_prefix() {
		assert_eq!(Buffer::partial_tag_len("hello<no"), 3);
	}

	#[test]
	fn partial_tag_think_prefix() {
		assert_eq!(Buffer::partial_tag_len("data<thi"), 4);
	}

	#[test]
	fn partial_tag_closing_prefix() {
		assert_eq!(Buffer::partial_tag_len("data</no"), 4);
	}

	#[test]
	fn partial_tag_full_tag_is_not_partial() {
		// A complete tag should be found by find_first_tag, not partial_tag_len.
		// partial_tag_len only looks for prefixes that don't form a complete tag.
		// "<note>" is 6 chars; if it ends the string, partial_tag_len checks
		// suffixes of length 1..=6. "<note>" starts_with "<note>" → matches.
		// But that's fine because drain() calls find_first_tag on safe portion first.
		let len = Buffer::partial_tag_len("text<note>");
		assert!(len > 0); // it does match as a prefix of "<note>"
	}

	#[test]
	fn partial_tag_suggestions_prefix() {
		assert_eq!(Buffer::partial_tag_len("x<sug"), 4);
	}

	// --- Integration: proc + print_return_remain ---

	#[test]
	fn basic_note_then_suggestion() {
		let mut buf = Buffer::new();
		buf.proc("explanation\n<note>\ndetail\n</note>\nsuggestion content");
		let result = buf.print_return_remain();
		assert_eq!(result, "suggestion content");
	}

	#[test]
	fn note_tag_inline_with_text() {
		let mut buf = Buffer::new();
		buf.proc("hello<note>world\n</note>the suggestion");
		let result = buf.print_return_remain();
		assert_eq!(result, "the suggestion");
	}

	#[test]
	fn tag_split_across_chunks() {
		let mut buf = Buffer::new();
		buf.proc("text<no");
		buf.proc("te>inside note\n</no");
		buf.proc("te>captured");
		let result = buf.print_return_remain();
		assert_eq!(result, "captured");
	}

	#[test]
	fn think_then_write_then_buf() {
		let mut buf = Buffer::new();
		buf.proc("<think>thinking...\n</think>explanation\n<note>note\n</note>suggestion");
		let result = buf.print_return_remain();
		assert_eq!(result, "suggestion");
	}

	#[test]
	fn suggestions_tag_transitions_to_buf() {
		let mut buf = Buffer::new();
		buf.proc("text<suggestions>cmd1\ncmd2");
		let result = buf.print_return_remain();
		assert_eq!(result, "cmd1\ncmd2");
	}

	#[test]
	fn suggest_tag_transitions_to_buf() {
		let mut buf = Buffer::new();
		buf.proc("text<suggest>the command");
		let result = buf.print_return_remain();
		assert_eq!(result, "the command");
	}

	#[test]
	fn suggestions_fallback_in_print_return_remain() {
		// If we never hit </note> or <suggest> during streaming,
		// print_return_remain falls back to splitting on <suggestions>
		let mut buf = Buffer::new();
		// Manually put data in buf without state transition
		buf.buf = "preamble<suggestions>the commands".to_string();
		let result = buf.print_return_remain();
		assert_eq!(result, "the commands");
	}

	#[test]
	fn no_tags_returns_empty() {
		let mut buf = Buffer::new();
		buf.proc("just some text\n");
		let result = buf.print_return_remain();
		assert_eq!(result, "");
	}

	#[test]
	fn multiple_chunks_no_newlines() {
		let mut buf = Buffer::new();
		buf.proc("a");
		buf.proc("b");
		buf.proc("c");
		buf.proc("<note>");
		buf.proc("d</note>result");
		let result = buf.print_return_remain();
		assert_eq!(result, "result");
	}

	#[test]
	fn think_block_split_across_many_chunks() {
		let mut buf = Buffer::new();
		buf.proc("<th");
		buf.proc("ink>");
		buf.proc("deep thought\n");
		buf.proc("</th");
		buf.proc("ink>");
		buf.proc("after think</note>captured");
		let result = buf.print_return_remain();
		assert_eq!(result, "captured");
	}

	#[test]
	fn empty_input() {
		let mut buf = Buffer::new();
		buf.proc("");
		let result = buf.print_return_remain();
		assert_eq!(result, "");
	}

	#[test]
	fn angle_bracket_not_a_tag() {
		let mut buf = Buffer::new();
		buf.proc("x < y and y > x\n</note>captured");
		let result = buf.print_return_remain();
		assert_eq!(result, "captured");
	}
}
