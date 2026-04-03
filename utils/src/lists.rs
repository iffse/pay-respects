// List of commands that are not runnable (e.g. TUI commands that hangs)
pub fn unrunnable_commands() -> Vec<&'static str> {
	vec!["vi", "vim", "nvim", "hx", "helix", "nano", "micro"]
}
