// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use colored::Colorize;
use crossterm::{
	cursor,
	event::{self, Event, KeyCode},
	execute,
	terminal::{self, ClearType},
};
use std::io::{Write, stderr};

use std::cmp::min;

pub fn select(
	prelude: &str,
	active_items: &[String],
	inactive_items: &[String],
) -> Result<usize, Box<dyn std::error::Error>> {
	terminal::enable_raw_mode()?;
	execute!(stderr(), cursor::Hide)?;
	eprint!("{}\r\n", prelude);

	let lines = {
		let mut lines = 0;
		for item in active_items {
			lines += item.lines().count();
		}
		lines
	};
	let total_lines = lines + prelude.lines().count();

	let shortcut_max = min(active_items.len(), 9);
	let mut current = 0;

	// Initial draw
	draw(active_items, inactive_items, current)?;

	loop {
		if let Event::Key(key) = event::read()? {
			match key.code {
				// Navigation keys
				KeyCode::Char('j') | KeyCode::Down => {
					current = (current + 1) % active_items.len();
				}
				KeyCode::Char('k') | KeyCode::Up => {
					current = if current == 0 {
						active_items.len() - 1
					} else {
						current - 1
					};
				}
				// Shortcut keys (1-9)
				KeyCode::Char(c) if c.is_ascii_digit() => {
					let idx = c.to_digit(10).unwrap() as usize - 1;
					if idx < shortcut_max {
						current = idx;
						break;
					}
				}
				// Quit keys
				KeyCode::Char('c') | KeyCode::Char('d') => {
					if key.modifiers.contains(event::KeyModifiers::CONTROL) {
						cleanup(total_lines)?;
						quit();
					}
				}
				KeyCode::Esc | KeyCode::Char('q') => {
					cleanup(total_lines)?;
					quit()
				}
				KeyCode::Enter => break,
				_ => {}
			}

			redraw(active_items, inactive_items, current, lines)?;
		}
	}

	// Cleanup
	cleanup(total_lines)?;
	terminal::disable_raw_mode()?;

	eprintln!("{}", active_items[current]);
	Ok(current)
}

pub fn select_simple(prelude: &str, items: &[String]) -> Result<usize, Box<dyn std::error::Error>> {
	terminal::enable_raw_mode()?;
	execute!(stderr(), cursor::Hide)?;
	eprint!("{}\r\n", prelude);

	let lines = {
		let mut lines = 0;
		for item in items {
			lines += item.lines().count();
		}
		lines
	};
	let total_lines = lines + prelude.lines().count();

	let shortcut_max = min(items.len(), 9);
	let mut current = 0;

	// Initial draw
	draw_simple(items, current)?;

	loop {
		if let Event::Key(key) = event::read()? {
			match key.code {
				// Navigation keys
				KeyCode::Char('j') | KeyCode::Down => {
					current = (current + 1) % items.len();
				}
				KeyCode::Char('k') | KeyCode::Up => {
					current = if current == 0 {
						items.len() - 1
					} else {
						current - 1
					};
				}
				// Shortcut keys (1-9)
				KeyCode::Char(c) if c.is_ascii_digit() => {
					let idx = c.to_digit(10).unwrap() as usize - 1;
					if idx < shortcut_max {
						current = idx;
						break;
					}
				}
				// Quit keys
				KeyCode::Char('c') | KeyCode::Char('d') => {
					if key.modifiers.contains(event::KeyModifiers::CONTROL) {
						cleanup(total_lines)?;
						quit();
					}
				}
				KeyCode::Esc | KeyCode::Char('q') => {
					cleanup(total_lines)?;
					quit()
				}
				KeyCode::Enter => break,
				_ => {}
			}

			redraw_simple(items, current, lines)?;
		}
	}

	// Cleanup
	cleanup(total_lines)?;
	terminal::disable_raw_mode()?;

	eprintln!("{}", items[current].cyan());
	Ok(current)
}

fn select_idx(idx: usize) -> String {
	if idx < 9 {
		format!("{}", idx + 1)
	} else {
		String::from("-")
	}
}

fn draw(
	active_items: &[String],
	inactive_items: &[String],
	selected: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	for (i, item) in active_items.iter().enumerate() {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		if i == selected {
			let prefix = format!("> {}) ", select_idx(i)).cyan().bold();
			let line = format!("{}{}", prefix, item);
			eprint!("{}\r\n", line);
		} else {
			let prefix = format!("  {}) ", select_idx(i)).cyan();
			let line = format!("{}{}", prefix, inactive_items.get(i).unwrap());
			eprint!("{}\r\n", line);
		}
	}
	stderr().flush()?;
	Ok(())
}

fn redraw(
	active_items: &[String],
	inactive_items: &[String],
	selected: usize,
	lines: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	execute!(stderr(), cursor::MoveUp(lines as u16))?;
	draw(active_items, inactive_items, selected)
}

fn draw_simple(items: &[String], selected: usize) -> Result<(), Box<dyn std::error::Error>> {
	for (i, item) in items.iter().enumerate() {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		if i == selected {
			let prefix = format!("> {}) ", select_idx(i)).cyan().bold();
			let line = format!("{}{}", prefix, item.cyan());
			eprint!("{}\r\n", line);
		} else {
			let prefix = format!("  {}) ", select_idx(i)).cyan();
			let line = format!("{}{}", prefix, item.normal());
			eprint!("{}\r\n", line);
		}
	}
	stderr().flush()?;
	Ok(())
}

fn redraw_simple(
	items: &[String],
	selected: usize,
	lines: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	execute!(stderr(), cursor::MoveUp(lines as u16))?;
	draw_simple(items, selected)
}

fn cleanup(lines: usize) -> Result<(), Box<dyn std::error::Error>> {
	execute!(stderr(), cursor::MoveUp(lines as u16))?;

	for _ in 0..lines {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		eprint!("\r\n");
	}

	execute!(stderr(), cursor::MoveUp(lines as u16))?;
	stderr().flush()?;

	execute!(stderr(), cursor::Show).unwrap();
	Ok(())
}

fn quit() -> ! {
	execute!(stderr(), cursor::Show).unwrap();
	terminal::disable_raw_mode().unwrap();
	let msg = "<Cancelled>".red();
	eprintln!("{}", msg);
	std::process::exit(0);
}
