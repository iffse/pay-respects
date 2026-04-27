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

#[derive(Default)]
struct Page {
	items: Vec<usize>,
	lines: usize,
}

#[cfg(target_os = "windows")]
use crossterm::event::KeyEventKind;

const MAX_ITEMS: usize = 10;

pub fn select(
	prelude: &str,
	active_items: &[String],
	inactive_items: &[String],
) -> Result<usize, Box<dyn std::error::Error>> {
	let height = terminal::size()?.1 as usize;
	let prelude_lines = prelude.lines().count();

	if height < prelude_lines + 1 {
		terminal::disable_raw_mode()?;
		eprintln!("Terminal height too small.");
		std::process::exit(1);
	}

	let pages = get_pages(active_items, height - prelude_lines - 2);

	terminal::enable_raw_mode()?;
	drain_input();

	execute!(stderr(), cursor::Hide)?;
	eprint!("{}\r\n", prelude);

	let mut current = 0;
	let mut page_idx = 0;
	let total_pages = pages.len();

	macro_rules! page_range {
		() => {
			pages[page_idx].items[0]..pages[page_idx].items[pages[page_idx].items.len() - 1] + 1
		};
	}
	macro_rules! page_len {
		() => {
			pages[page_idx].items.len()
		};
	}
	macro_rules! next_page {
		() => {
			page_idx = (page_idx + 1) % pages.len();
			current = 0;
		};
	}
	macro_rules! prev_page {
		() => {
			page_idx = if page_idx == 0 {
				pages.len() - 1
			} else {
				page_idx - 1
			};
			current = pages[page_idx].items.len() - 1;
		};
	}
	macro_rules! clear_lines {
		() => {
			if pages.len() == 1 {
				pages[page_idx].lines
			} else {
				pages[page_idx].lines + 1
			}
		};
	}

	// Initial draw
	draw(
		&active_items[page_range!()],
		&inactive_items[page_range!()],
		current,
		page_idx,
		total_pages,
	)?;

	loop {
		if let Event::Key(key) = event::read()? {
			// somehow windows receives two events
			#[cfg(target_os = "windows")]
			if key.kind != KeyEventKind::Press {
				continue;
			}

			let clear_lines = clear_lines!();
			match key.code {
				// Navigation keys
				KeyCode::Char('j') | KeyCode::Down => {
					// current = (current + 1) % active_items.len();
					if current + 1 >= page_len!() {
						next_page!();
					} else {
						current += 1;
					}
				}
				KeyCode::Char('k') | KeyCode::Up => {
					if current == 0 {
						prev_page!();
					} else {
						current -= 1
					};
				}
				// Page navigation keys
				KeyCode::Char('f') | KeyCode::PageDown => {
					next_page!();
				}
				KeyCode::Char('b') | KeyCode::PageUp => {
					prev_page!();
				}
				// Shortcut keys (1-0)
				KeyCode::Char(c) if c.is_ascii_digit() => {
					if c == '0' {
						if page_len!() == 10 {
							current = MAX_ITEMS - 1;
						}
						break;
					}
					let idx = c.to_digit(10).unwrap() as usize - 1;
					if idx < pages[page_idx].items.len() {
						current = idx;
						break;
					}
				}
				// Quit keys
				KeyCode::Char('c') | KeyCode::Char('d') => {
					if key.modifiers.contains(event::KeyModifiers::CONTROL) {
						cleanup(pages[page_idx].lines)?;
						quit();
					}
				}
				KeyCode::Esc | KeyCode::Char('q') => {
					cleanup(clear_lines!() + prelude_lines)?;
					quit()
				}
				KeyCode::Enter => break,
				_ => {}
			}

			redraw(
				&active_items[page_range!()],
				&inactive_items[page_range!()],
				current,
				clear_lines,
				page_idx,
				total_pages,
			)?;
		}
		drain_input();
	}

	// Cleanup
	cleanup(clear_lines!() + prelude_lines)?;
	terminal::disable_raw_mode()?;

	Ok(pages[page_idx].items[current])
}

pub fn select_simple(prelude: &str, items: &[String]) -> Result<usize, Box<dyn std::error::Error>> {
	let active_items = items
		.iter()
		.map(|s| s.cyan().to_string())
		.collect::<Vec<String>>();
	select(prelude, &active_items, items)
}

fn select_idx(idx: usize) -> String {
	let idx = idx + 1;
	if idx < MAX_ITEMS {
		format!("{}", idx)
	} else if idx == MAX_ITEMS {
		String::from("0")
	} else {
		String::from("-")
	}
}

fn get_pages(active_items: &[String], max_height: usize) -> Vec<Page> {
	let mut pages = Vec::new();
	let mut current_page = Page::default();
	for (idx, item) in active_items.iter().enumerate() {
		let item_lines = item.lines().count();
		if item_lines > max_height {
			eprintln!("An item is too long to fit in the terminal.");
			std::process::exit(1);
		}
		if current_page.lines + item_lines > max_height || current_page.items.len() >= MAX_ITEMS {
			pages.push(current_page);
			current_page = Page::default();
		}
		current_page.items.push(idx);
		current_page.lines += item_lines;
	}
	if !current_page.items.is_empty() {
		pages.push(current_page);
	}
	pages
}

fn print(str: &str) {
	// TODO: Trimming long lines to fit terminal width
	// Not working well due to color codes.
	eprint!("{}\r\n", str);
}

fn draw(
	active_items: &[String],
	inactive_items: &[String],
	selected: usize,
	current_page: usize,
	total_pages: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	for (i, item) in active_items.iter().enumerate() {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		if i == selected {
			let prefix = format!("> {}) ", select_idx(i)).cyan().bold();
			let line = format!("{}{}", prefix, item);
			print(&line);
		} else {
			let prefix = format!("  {}) ", select_idx(i)).cyan();
			let line = format!("{}{}", prefix, inactive_items.get(i).unwrap());
			print(&line);
		}
	}
	if total_pages > 1 {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		let page_info = format!("[{}/{}]", current_page + 1, total_pages)
			.cyan()
			.to_string();
		print(&page_info);
	}
	stderr().flush()?;
	Ok(())
}

fn redraw(
	active_items: &[String],
	inactive_items: &[String],
	selected: usize,
	lines: usize,
	currrent_page: usize,
	total_pages: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	execute!(stderr(), cursor::MoveUp(lines as u16))?;
	for _ in 0..lines {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		eprint!("\r\n");
	}
	execute!(stderr(), cursor::MoveUp(lines as u16))?;
	draw(
		active_items,
		inactive_items,
		selected,
		currrent_page,
		total_pages,
	)
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

fn drain_input() {
	#[cfg(target_os = "windows")]
	while event::poll(std::time::Duration::from_millis(10)).unwrap() {
		event::read().unwrap();
	}
}
