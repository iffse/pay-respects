const DEBUG_LEVEL: usize = 0;
const LOG_TO_FILE: bool = false;

use crate::strings::{log_plain, log_string};

/// Logs debug messages in the way that I will never ship them to production by accident again
#[allow(unused)]
pub fn dlog(debug_level: usize, message: &str) {
	#[cfg(not(debug_assertions))]
	return;

	if DEBUG_LEVEL < debug_level {
		return;
	}

	if LOG_TO_FILE {
		use std::fs::OpenOptions;
		use std::io::Write;

		let message = log_plain(debug_level, message);

		if let Ok(mut file) = OpenOptions::new()
			.append(true)
			.create(true)
			.open("pay-respects.log")
		{
			let _ = writeln!(file, "{}", message);
		}
	} else {
		let message = log_string(debug_level, message);
		eprintln!("{}", message);
	}
}
