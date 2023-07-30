mod corrections;
mod shell;
mod style;

fn main() {
	std::env::set_var("LC_ALL", "C");

	let corrected_command = corrections::correct_command();
	if let Some(corrected_command) = corrected_command {
		corrections::confirm_correction(&corrected_command);
	} else {
		println!("No correction found.");
	}
}
