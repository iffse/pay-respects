// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use colored::{Color, ColoredString, Colorize};
use serde::Deserialize;

use crate::strings::print_warning;

/// Options for the selection interface, shared by every crate that draws one.
#[derive(Debug, Clone, Copy)]
pub struct UiConfig {
	/// Print the message above the list.
	pub prelude: bool,
	/// Print the line with the key hints.
	pub hint: bool,
	/// Put the selected suggestion back into the prompt for editing. When
	/// false, or when the shell cannot do it, the command is opened in the
	/// editor instead.
	pub prompt_edit: bool,
	pub colors: Colors,
}

impl Default for UiConfig {
	fn default() -> Self {
		Self {
			prelude: true,
			hint: true,
			prompt_edit: true,
			colors: Colors::default(),
		}
	}
}

impl UiConfig {
	pub fn merge(&mut self, reader: UiConfigReader) {
		if let Some(prelude) = reader.prelude {
			self.prelude = prelude;
		}
		if let Some(hint) = reader.hint {
			self.hint = hint;
		}
		if let Some(prompt_edit) = reader.prompt_edit {
			self.prompt_edit = prompt_edit;
		}
	}
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct UiConfigReader {
	pub prelude: Option<bool>,
	pub hint: Option<bool>,
	pub prompt_edit: Option<bool>,
}

/// Text colors for the interface. Values accept color names (`red`, `bright
/// blue`) and hex codes (`#ff0000`).
#[derive(Debug, Clone, Copy)]
pub struct Colors {
	/// Parts to correct and error messages.
	pub wrong: Color,
	/// Confirmations and successful states.
	pub correct: Color,
	/// Unchanged parts and interface elements.
	pub accent: Color,
	/// Normal text. Unset keeps the terminal's default foreground.
	pub normal: Option<Color>,
}

impl Default for Colors {
	fn default() -> Self {
		Self {
			wrong: Color::Red,
			correct: Color::Green,
			accent: Color::Blue,
			normal: None,
		}
	}
}

impl Colors {
	pub fn merge(&mut self, reader: ColorsReader) {
		set_color(&mut self.wrong, reader.wrong.as_deref(), "wrong");
		set_color(&mut self.correct, reader.correct.as_deref(), "correct");
		set_color(&mut self.accent, reader.accent.as_deref(), "accent");
		if let Some(normal) = reader.normal.as_deref() {
			set_normal(&mut self.normal, normal);
		}
	}

	/// Applies the normal text color, keeping the terminal default when unset.
	pub fn base(&self, string: &str) -> ColoredString {
		match self.normal {
			Some(color) => string.color(color),
			None => string.normal(),
		}
	}
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ColorsReader {
	pub wrong: Option<String>,
	pub correct: Option<String>,
	pub accent: Option<String>,
	pub normal: Option<String>,
}

fn parse_color(value: &str) -> Option<Color> {
	value.trim().parse().ok()
}

fn set_color(target: &mut Color, value: Option<&str>, name: &str) {
	let Some(value) = value else {
		return;
	};
	match parse_color(value) {
		Some(color) => *target = color,
		None => print_warning(&format!(
			"Invalid color for `colors.{}`: `{}`. Using the default.",
			name, value
		)),
	}
}

fn set_normal(target: &mut Option<Color>, value: &str) {
	let value = value.trim();
	if value.is_empty()
		|| value.eq_ignore_ascii_case("none")
		|| value.eq_ignore_ascii_case("default")
	{
		*target = None;
		return;
	}
	match parse_color(value) {
		Some(color) => *target = Some(color),
		None => print_warning(&format!(
			"Invalid color for `colors.normal`: `{}`. Using the default.",
			value
		)),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Deserialize)]
	struct Config {
		ui: Option<UiConfigReader>,
		colors: Option<ColorsReader>,
	}

	fn parse(content: &str) -> UiConfig {
		let reader: Config = toml::from_str(content).unwrap();
		let mut ui = UiConfig::default();
		if let Some(ui_reader) = reader.ui {
			ui.merge(ui_reader);
		}
		if let Some(colors) = reader.colors {
			ui.colors.merge(colors);
		}
		ui
	}

	#[test]
	fn reads_visibility_flags() {
		let ui = parse(
			"
[ui]
prelude = false
hint = false
",
		);

		assert!(!ui.prelude);
		assert!(!ui.hint);
		assert!(ui.prompt_edit);
	}

	#[test]
	fn can_prefer_the_editor() {
		let ui = parse(
			"
[ui]
prompt_edit = false
",
		);

		assert!(!ui.prompt_edit);
	}

	#[test]
	fn defaults_keep_the_interface_visible() {
		let ui = parse("");

		assert!(ui.prelude);
		assert!(ui.hint);
		assert!(ui.prompt_edit);
		assert_eq!(ui.colors.wrong, Color::Red);
		assert_eq!(ui.colors.correct, Color::Green);
		assert_eq!(ui.colors.accent, Color::Blue);
		assert_eq!(ui.colors.normal, None);
	}

	#[test]
	fn reads_color_names_and_hex_codes() {
		let ui = parse(
			r##"
[colors]
wrong = "bright red"
correct = "#00ff00"
accent = "magenta"
normal = "white"
"##,
		);

		assert_eq!(ui.colors.wrong, Color::BrightRed);
		assert_eq!(ui.colors.correct, Color::TrueColor { r: 0, g: 255, b: 0 });
		assert_eq!(ui.colors.accent, Color::Magenta);
		assert_eq!(ui.colors.normal, Some(Color::White));
	}

	#[test]
	fn normal_can_be_reset() {
		let ui = parse(
			r#"
[colors]
normal = "none"
"#,
		);

		assert_eq!(ui.colors.normal, None);
	}

	#[test]
	fn invalid_colors_keep_the_defaults() {
		let ui = parse(
			r##"
[colors]
wrong = "nope"
accent = "#zzz"
"##,
		);

		assert_eq!(ui.colors.wrong, Color::Red);
		assert_eq!(ui.colors.accent, Color::Blue);
	}
}
