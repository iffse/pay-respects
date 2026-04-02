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
	eprint!("{}\r\n", prelude);

	let lines = {
		let mut lines = 0;
		for item in active_items {
			lines += item.lines().count();
		}
		lines
	};
	let total_lines = lines + prelude.lines().count();

	// TODO: support for pagination if there are more than 9 items
	if lines > 9 {
		return Err(format!("Too many items ({}). Pagnation not implemented.", lines).into());
	}

	let shortcut_max = min(active_items.len(), 9);
	let mut current = 0;

	let mut stderr = stderr();

	// Initial draw
	draw(&mut stderr, active_items, inactive_items, current)?;

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
						cleanup(&mut stderr, total_lines)?;
						quit();
					}
				}
				KeyCode::Esc | KeyCode::Char('q') => {
					cleanup(&mut stderr, total_lines)?;
					quit()
				}
				KeyCode::Enter => break,
				_ => {}
			}

			redraw(&mut stderr, active_items, inactive_items, current, lines)?;
		}
	}

	// Cleanup
	cleanup(&mut stderr, total_lines)?;
	terminal::disable_raw_mode()?;

	eprintln!("{}", active_items[current]);
	Ok(current)
}

pub fn select_simple(prelude: &str, items: &[String]) -> Result<usize, Box<dyn std::error::Error>> {
	terminal::enable_raw_mode()?;
	eprint!("{}\r\n", prelude);

	let lines = {
		let mut lines = 0;
		for item in items {
			lines += item.lines().count();
		}
		lines
	};
	let total_lines = lines + prelude.lines().count();

	// TODO: support for pagination if there are more than 9 items
	if lines > 9 {
		return Err(format!("Too many items ({}). Pagnation not implemented.", lines).into());
	}

	let shortcut_max = min(items.len(), 9);
	let mut current = 0;

	let mut stderr = stderr();

	// Initial draw
	draw_simple(&mut stderr, items, current)?;

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
						cleanup(&mut stderr, total_lines)?;
						quit();
					}
				}
				KeyCode::Esc | KeyCode::Char('q') => {
					cleanup(&mut stderr, total_lines)?;
					quit()
				}
				KeyCode::Enter => break,
				_ => {}
			}

			redraw_simple(&mut stderr, items, current, lines)?;
		}
	}

	// Cleanup
	cleanup(&mut stderr, total_lines)?;
	terminal::disable_raw_mode()?;

	eprintln!("{}", items[current]);
	Ok(current)
}

fn draw(
	stderr: &mut std::io::Stderr,
	active_items: &[String],
	inactive_items: &[String],
	selected: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	for (i, item) in active_items.iter().enumerate() {
		execute!(stderr, terminal::Clear(ClearType::CurrentLine))?;
		if i == selected {
			let prefix = format!("> {}) ", i + 1).cyan().bold();
			let line = format!("{}{}", prefix, item);
			eprint!("{}\r\n", line);
		} else {
			let prefix = format!("  {}) ", i + 1).cyan();
			let line = format!("{}{}", prefix, inactive_items.get(i).unwrap());
			eprint!("{}\r\n", line);
		}
	}
	stderr.flush()?;
	Ok(())
}

fn redraw(
	stderr: &mut std::io::Stderr,
	active_items: &[String],
	inactive_items: &[String],
	selected: usize,
	lines: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	execute!(stderr, cursor::MoveUp(lines as u16))?;
	draw(stderr, active_items, inactive_items, selected)
}

fn draw_simple(
	stderr: &mut std::io::Stderr,
	items: &[String],
	selected: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	for (i, item) in items.iter().enumerate() {
		execute!(stderr, terminal::Clear(ClearType::CurrentLine))?;
		if i == selected {
			let prefix = format!("> {}) ", i + 1).cyan().bold();
			let line = format!("{}{}", prefix, item.cyan());
			eprint!("{}\r\n", line);
		} else {
			let prefix = format!("  {}) ", i + 1).cyan();
			let line = format!("{}{}", prefix, item.normal());
			eprint!("{}\r\n", line);
		}
	}
	stderr.flush()?;
	Ok(())
}

fn redraw_simple(
	stderr: &mut std::io::Stderr,
	items: &[String],
	selected: usize,
	lines: usize,
) -> Result<(), Box<dyn std::error::Error>> {
	execute!(stderr, cursor::MoveUp(lines as u16))?;
	draw_simple(stderr, items, selected)
}

fn cleanup(stderr: &mut std::io::Stderr, lines: usize) -> Result<(), Box<dyn std::error::Error>> {
	execute!(stderr, cursor::MoveUp(lines as u16))?;

	for _ in 0..lines {
		execute!(stderr, terminal::Clear(ClearType::CurrentLine))?;
		eprint!("\r\n");
	}

	execute!(stderr, cursor::MoveUp(lines as u16))?;
	stderr.flush()?;

	Ok(())
}

fn quit() -> ! {
	terminal::disable_raw_mode().unwrap();
	let msg = "<Cancelled>".red();
	eprintln!("{}", msg);
	std::process::exit(0);
}
