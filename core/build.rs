fn main() {
	// recompile when rules are updated
	println!("cargo::rerun-if-changed=./rules/")
}
