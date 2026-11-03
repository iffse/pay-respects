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
use pay_respects_utils::strings::print_error;
use std::io::{Write, stderr};

#[derive(Default)]
struct Page {
	items: Vec<usize>,
	lines: usize,
}

#[cfg(target_os = "windows")]
use crossterm::event::KeyEventKind;

const MAX_ITEMS: usize = 10;
/// Rows kept free below the list: one for the page indicator and one so that
/// printing the newline after the last row never scrolls the terminal.
const BOTTOM_MARGIN: usize = 2;

pub fn select(
	prelude: &str,
	active_items: &[String],
	inactive_items: &[String],
) -> Result<usize, Box<dyn std::error::Error>> {
	let prelude_lines = prelude_height(prelude);
	let mut size = terminal::size()?;
	let mut list_lines = available_height(size.1, prelude_lines);

	if list_lines == 0 {
		print_error("Terminal height too small.");
		std::process::exit(1);
	}
	if active_items
		.iter()
		.any(|item| item.lines().count() > list_lines)
	{
		print_error("An item is too long to fit in the terminal.");
		std::process::exit(1);
	}

	let mut pages = get_pages(active_items, list_lines);
	let active_items = add_padding(active_items, 5);
	let inactive_items = add_padding(inactive_items, 5);

	terminal::enable_raw_mode()?;
	execute!(stderr(), terminal::DisableLineWrap)?;
	drain_input();

	execute!(stderr(), cursor::Hide)?;
	// The prelude is the anchor of the whole UI: on a resize the cursor is
	// restored here and everything below is redrawn, since the terminal may
	// have scrolled the previous frame out of place.
	execute!(stderr(), cursor::SavePosition)?;
	eprint!("{}\r\n", prelude);

	let mut current = 0;
	let mut page_idx = 0;

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

	// Initial draw
	let mut drawn = draw(
		&active_items[page_range!()],
		&inactive_items[page_range!()],
		current,
		page_idx,
		pages.len(),
		list_lines,
	)?;

	loop {
		// Resizes are handled below by comparing the terminal size, which also
		// catches the events swallowed while draining input on windows.
		if let Event::Key(key) = event::read()? {
			// somehow windows receives two events
			#[cfg(target_os = "windows")]
			if key.kind != KeyEventKind::Press {
				continue;
			}

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
						if page_len!() == MAX_ITEMS {
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
				KeyCode::Char('c') | KeyCode::Char('d')
					if key.modifiers.contains(event::KeyModifiers::CONTROL) =>
				{
					cleanup(drawn + prelude_lines)?;
					quit();
				}
				KeyCode::Esc | KeyCode::Char('q') => {
					cleanup(drawn + prelude_lines)?;
					quit()
				}
				KeyCode::Enter => break,
				_ => {}
			}

			drawn = redraw(
				&active_items[page_range!()],
				&inactive_items[page_range!()],
				current,
				drawn,
				page_idx,
				pages.len(),
				list_lines,
			)?;
		}
		drain_input();

		let new_size = terminal::size()?;
		if new_size != size {
			size = new_size;
			list_lines = available_height(size.1, prelude_lines);
			if list_lines == 0 {
				continue;
			}

			// Keep the cursor on the same entry after repagination.
			let selected = pages[page_idx].items[current];
			let new_pages = get_pages(&active_items, list_lines);
			if new_pages.is_empty() {
				continue;
			}
			pages = new_pages;
			(page_idx, current) = locate_item(&pages, selected).unwrap_or((0, 0));

			// The frame may have been reflowed or scrolled by the terminal, so
			// relative cursor movements are no longer reliable. Go back to the
			// saved anchor and redraw the prelude and the list from there.
			execute!(stderr(), cursor::RestorePosition)?;
			execute!(stderr(), terminal::Clear(ClearType::FromCursorDown))?;
			execute!(stderr(), cursor::SavePosition)?;
			eprint!("{}\r\n", prelude);
			drawn = draw(
				&active_items[page_range!()],
				&inactive_items[page_range!()],
				current,
				page_idx,
				pages.len(),
				list_lines,
			)?;
		}
	}

	// Cleanup
	cleanup(drawn + prelude_lines)?;
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

/// Number of rows occupied by the prelude, including the newline printed after
/// it.
fn prelude_height(prelude: &str) -> usize {
	prelude.matches('\n').count() + 1
}

/// Rows available for the item list.
fn available_height(terminal_height: u16, prelude_lines: usize) -> usize {
	(terminal_height as usize).saturating_sub(prelude_lines + BOTTOM_MARGIN)
}

fn get_pages(active_items: &[String], max_height: usize) -> Vec<Page> {
	let max_height = max_height.max(1);
	let mut pages = Vec::new();
	let mut current_page = Page::default();

	for (idx, item) in active_items.iter().enumerate() {
		let item_lines = item.lines().count().max(1);
		let page_full =
			current_page.lines + item_lines > max_height || current_page.items.len() >= MAX_ITEMS;

		// An item taller than the terminal gets a page of its own.
		if page_full && !current_page.items.is_empty() {
			pages.push(std::mem::take(&mut current_page));
		}

		current_page.items.push(idx);
		current_page.lines += item_lines;
	}

	if !current_page.items.is_empty() {
		pages.push(current_page);
	}
	pages
}

fn locate_item(pages: &[Page], item: usize) -> Option<(usize, usize)> {
	pages.iter().enumerate().find_map(|(page_idx, page)| {
		page.items
			.iter()
			.position(|&candidate| candidate == item)
			.map(|current| (page_idx, current))
	})
}

/// Prints `str`, at most `max_lines` rows of it, and returns the number of rows
/// written.
fn print(str: &str, max_lines: usize) -> usize {
	let mut lines = 0;
	for line in str.split('\n').take(max_lines) {
		let line = line.strip_suffix('\r').unwrap_or(line);
		eprint!("{} \r\n", line);
		lines += 1;
	}
	lines
}

fn draw(
	active_items: &[String],
	inactive_items: &[String],
	selected: usize,
	current_page: usize,
	total_pages: usize,
	max_lines: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
	let mut lines = 0;

	for (i, item) in active_items.iter().enumerate() {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		let line = if i == selected {
			let prefix = format!("> {}) ", select_idx(i)).cyan().bold();
			format!("{}{}", prefix, item)
		} else {
			let prefix = format!("  {}) ", select_idx(i)).cyan();
			format!("{}{}", prefix, inactive_items.get(i).unwrap())
		};
		lines += print(&line, max_lines.saturating_sub(lines));
		if lines >= max_lines {
			break;
		}
	}

	if total_pages > 1 {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		let page_info = format!("[{}/{}]", current_page + 1, total_pages)
			.cyan()
			.to_string();
		// The page indicator gets one row of its own, reserved by
		// `BOTTOM_MARGIN`.
		lines += print(&page_info, max_lines + 1 - lines);
	}

	stderr().flush()?;
	Ok(lines)
}

fn redraw(
	active_items: &[String],
	inactive_items: &[String],
	selected: usize,
	drawn_lines: usize,
	current_page: usize,
	total_pages: usize,
	max_lines: usize,
) -> Result<usize, Box<dyn std::error::Error>> {
	clear_lines(drawn_lines)?;
	draw(
		active_items,
		inactive_items,
		selected,
		current_page,
		total_pages,
		max_lines,
	)
}

fn clear_lines(lines: usize) -> Result<(), Box<dyn std::error::Error>> {
	if lines == 0 {
		return Ok(());
	}

	execute!(stderr(), cursor::MoveUp(lines as u16))?;

	for _ in 0..lines {
		execute!(stderr(), terminal::Clear(ClearType::CurrentLine))?;
		eprint!("\r\n");
	}

	execute!(stderr(), cursor::MoveUp(lines as u16))?;
	Ok(())
}

fn cleanup(lines: usize) -> Result<(), Box<dyn std::error::Error>> {
	clear_lines(lines)?;
	stderr().flush()?;

	execute!(stderr(), cursor::Show)?;
	Ok(())
}

fn quit() -> ! {
	execute!(stderr(), cursor::Show).unwrap();
	execute!(stderr(), terminal::EnableLineWrap).unwrap();
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

fn add_padding(vec: &[String], width: usize) -> Vec<String> {
	let spaces = " ".repeat(width);
	let from = '\n';
	let to = format!("\r\n{}", spaces);
	vec.iter().map(|s| s.replace(from, &to)).collect()
}

#[cfg(test)]
mod tests {
	use super::*;

	fn items(count: usize) -> Vec<String> {
		(0..count).map(|i| format!("item {}", i)).collect()
	}

	#[test]
	fn pages_split_on_height() {
		let pages = get_pages(&items(5), 2);
		assert_eq!(pages.len(), 3);
		assert_eq!(pages[0].items, vec![0, 1]);
		assert_eq!(pages[1].items, vec![2, 3]);
		assert_eq!(pages[2].items, vec![4]);
	}

	#[test]
	fn pages_cap_at_max_items() {
		let pages = get_pages(&items(MAX_ITEMS + 2), 100);
		assert_eq!(pages.len(), 2);
		assert_eq!(pages[0].items.len(), MAX_ITEMS);
		assert_eq!(pages[1].items, vec![MAX_ITEMS, MAX_ITEMS + 1]);
	}

	#[test]
	fn multiline_items_take_their_rows() {
		let items = vec![
			String::from("a\nb"),
			String::from("c"),
			String::from("d\ne\nf"),
			String::from("g"),
		];
		let pages = get_pages(&items, 4);
		assert_eq!(pages.len(), 2);
		assert_eq!(pages[0].items, vec![0, 1]);
		assert_eq!(pages[1].items, vec![2, 3]);
	}

	#[test]
	fn tall_item_gets_its_own_page() {
		let items = vec![
			String::from("short"),
			String::from("a\nb\nc\nd"),
			String::from("short"),
		];
		let pages = get_pages(&items, 2);
		assert_eq!(pages.len(), 3);
		assert_eq!(pages[1].items, vec![1]);
	}

	#[test]
	fn layout_is_stable_across_resizes() {
		let items = items(7);
		for height in 1..20 {
			let pages = get_pages(&items, height);
			let indexed: Vec<usize> = pages.iter().flat_map(|p| p.items.clone()).collect();
			assert_eq!(indexed, (0..items.len()).collect::<Vec<_>>());
		}
	}

	#[test]
	fn locate_item_finds_item_after_repagination() {
		let items = items(5);
		let pages = get_pages(&items, 2);
		assert_eq!(locate_item(&pages, 3), Some((1, 1)));
		assert_eq!(locate_item(&pages, 4), Some((2, 0)));
		assert_eq!(locate_item(&pages, 9), None);
	}
}
