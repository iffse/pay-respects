#[cfg(debug_assertions)]
use crate::strings::{log_plain, log_string};
#[cfg(debug_assertions)]
use std::path::PathBuf;
#[cfg(debug_assertions)]
use std::sync::OnceLock;

#[cfg(debug_assertions)]
enum Destination {
	Off,
	Stderr,
	File(PathBuf),
}

#[cfg(debug_assertions)]
struct Config {
	destination: Destination,
	level: usize,
}

#[cfg(debug_assertions)]
fn config() -> &'static Config {
	static CONFIG: OnceLock<Config> = OnceLock::new();

	CONFIG.get_or_init(|| {
		let destination = match std::env::var("_PR_LOG").as_deref() {
			Ok("stderr") | Ok("-") => Destination::Stderr,
			Ok("1") => Destination::File(PathBuf::from("pay-respects.log")),
			Ok(path) if !path.is_empty() && path != "0" && path != "off" => {
				Destination::File(PathBuf::from(path))
			}
			_ => Destination::Off,
		};

		let level = std::env::var("_PR_LOG_LEVEL")
			.ok()
			.and_then(|level| level.parse().ok())
			.unwrap_or(5);

		Config { destination, level }
	})
}

#[cfg(debug_assertions)]
pub fn enabled(level: usize) -> bool {
	!matches!(config().destination, Destination::Off) && level <= config().level
}

/// Logs debug messages to stderr or a file selected by `_PR_LOG`.
#[cfg(debug_assertions)]
pub fn dlog(level: usize, message: &str) {
	if !enabled(level) {
		return;
	}

	match &config().destination {
		Destination::Off => {}
		Destination::Stderr => {
			eprintln!("{}", log_string(level, message));
		}
		Destination::File(path) => {
			use std::fs::OpenOptions;
			use std::io::Write;

			if let Ok(mut file) = OpenOptions::new().append(true).create(true).open(path) {
				let _ = writeln!(file, "{}", log_plain(level, message));
			}
		}
	}
}

#[macro_export]
macro_rules! dlog {
	($level:expr, $($arg:tt)*) => {{
		#[cfg(debug_assertions)]
		{
			let level = $level;
			if $crate::log::enabled(level) {
				$crate::log::dlog(level, &format!($($arg)*));
			}
		}
	}};
}
