pub struct Init {
	pub shell: String,
	pub binary_path: String,
	pub alias: String,
	pub cnf: bool,
}

impl Init {
	pub fn new() -> Init {
		Init {
			shell: String::from(""),
			binary_path: String::from(""),
			alias: String::from("f"),
			cnf: true,
		}
	}
}
