use std::io::stderr;
use std::process::Command;
use std::process::Stdio;
use crate::shell::Data;

pub fn get_package_manager(data: &mut Data) -> Option<String> {
	let package_managers = vec!["pacman"];

	for package_manager in package_managers {
		if data.has_executable(package_manager) {
			return Some(package_manager.to_string());
		}
	}
	None
}

pub fn get_packages(shell: &str, package_manager: &str, executable: &str) -> Option<Vec<String>> {
	match package_manager {
		"pacman" => {
			let result = Command::new(shell)
				.arg("-c")
				.arg(format!("pacman -Fq /usr/bin/{}", executable))
				.output()
				.expect("failed to execute process");
			if result.status.success() {
				let output = String::from_utf8_lossy(&result.stdout)
					.lines()
					.map(|line| line.split_whitespace().next().unwrap().to_string())
					.collect();
				Some(output)
			} else {
				None
			}
		}
		_ => unreachable!("Unsupported package manager"),
	}
}

pub fn install_package(shell: &str, package_manager: &str, package: &str) -> bool {
	match package_manager {
		"pacman" => Command::new(shell)
			.arg("-c")
			.arg(format!("sudo pacman -S {}", package))
			.stdout(stderr())
			.stderr(Stdio::inherit())
			.spawn()
			.expect("failed to execute process")
			.wait()
			.unwrap()
			.success(),
		_ => unreachable!("Unsupported package manager"),
	}
}
