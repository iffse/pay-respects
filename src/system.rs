use crate::shell::{Data, command_output};
use std::io::stderr;
use std::process::Command;
use std::process::Stdio;

pub fn get_package_manager(data: &mut Data) -> Option<String> {
	let package_managers = vec![
		"apt",
		"dnf",
		"emerge",
		"nix-env",
		"pacman",
		// "pkg",
		// "yum",
		// "zypper",
	];

	for package_manager in package_managers {
		if data.has_executable(package_manager) {
			return Some(package_manager.to_string());
		}
	}
	None
}

pub fn get_packages(data: &mut Data, package_manager: &str, executable: &str) -> Option<Vec<String>> {
	let shell = &data.shell.clone();
	match package_manager {
		"apt" => {
			if !data.has_executable("dpkg") {
				return None;
			}
			let result = command_output(shell, &format!("dpkg -S '*/bin/{}'", executable));
			let packages: Vec<String> = result
				.lines()
				.map(|line| line[..line.find(':').unwrap()].to_string())
				.collect();
			
			if packages.is_empty() {
				None
			} else {
				Some(packages)
			}
		},
		"dnf" => {
			let result = command_output(shell, &format!("dnf provides {}", executable));
			let packages: Vec<String> = result
				.lines()
				.map(|line| line.split_whitespace().next().unwrap().to_string())
				.collect();
			if packages.is_empty() {
				None
			} else {
				Some(packages)
			}
		},
		"emerge" => {
			if !data.has_executable("e-file") {
				return None;
			}
			let result = command_output(shell, &format!("e-file /usr/bin/{}", executable));
			let mut packages = vec![];
			for line in result.lines() {
				if !line.starts_with(" ") {
					packages.push(line.to_string());
				}
			}
			if packages.is_empty() {
				None
			} else {
				Some(packages)
			}
		},
		"nix-env" => {
			if !data.has_executable("nix-locate") {
				return None;
			}
			let result = command_output(shell, &format!("nix-locate /usr/bin/{}", executable));
			let packages: Vec<String> = result
				.lines()
				.map(|line| line.split_whitespace().next().unwrap().trim_end_matches(".out").to_string())
				.collect();
			if packages.is_empty() {
				None
			} else {
				Some(packages)
			}
		}
		"pacman" => {
			let result = if data.has_executable("pkgfile") {
				command_output(shell, &format!("pkgfile -b {}", executable))
			} else {
				command_output(shell, &format!("pacman -Fq /usr/bin/{}", executable))
			};
			let packages: Vec<String> = result
				.lines()
				.map(|line| line.split_whitespace().next().unwrap().to_string())
				.collect();
			if packages.is_empty() {
				None
			} else {
				Some(packages)
			}
		},
		_ => unreachable!("Unsupported package manager"),
	}
}

pub fn install_package(shell: &str, package_manager: &str, package: &str) -> bool {
	let install = match package_manager {
		"apt" => format!("sudo apt install {}", package),
		"dnf" => format!("sudo dnf install {}", package),
		"emerge" => format!("sudo emerge {}", package),
		"nix-env" => format!("nix-env -iA {}", package),
		"pacman" => format!("sudo pacman -S {}", package),
		"pkg" => format!("sudo pkg install {}", package),
		"yum" => format!("sudo yum install {}", package),
		"zypper" => format!("sudo zypper install {}", package),
		_ => unreachable!("Unsupported package manager"),
	};

	let result = Command::new(shell)
		.arg("-c")
		.arg(install)
		.stdout(Stdio::null())
		.stderr(Stdio::null())
		.status()
		.expect("failed to execute process");

	result.success()
}
