pub struct Init {
	pub shell: String,
	pub shell_type: String,
	pub binary_path: String,
	pub alias: String,
	pub cnf: bool,
	pub cmd_prefix: Option<String>,
}

impl Init {
	pub fn new() -> Init {
		Init {
			shell: String::from(""),
			shell_type: String::from(""),
			binary_path: String::from(""),
			alias: String::from("f"),
			cnf: true,
			cmd_prefix: None,
		}
	}
}
