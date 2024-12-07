use crate::shell::{Data, command_output, elevate};
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
			if result.is_empty() {
				return None
			}
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
			if result.is_empty() {
				return None
			}
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
			if result.is_empty() {
				return None
			}
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
			let result = command_output(shell, &format!("nix-locate 'bin/{}'", executable));
			if result.is_empty() {
				return None
			}
			let packages: Vec<String> = result
				.lines()
				.map(|line| line.split_whitespace().next().unwrap().to_string())
				.collect();
			if packages.is_empty() {
				None
			} else {
				Some(packages)
			}
		}
		"pacman" => {
			// somehow it tries to always update, so very slow
			// let result = if data.has_executable("pkgfile") {
			// 	command_output(shell, &format!("pkgfile -b {}", executable))
			let result = command_output(shell, &format!("pacman -Fq /usr/bin/{}", executable));
			if result.is_empty() {
				return None
			}
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

pub fn install_package(data: &mut Data, package_manager: &str, package: &str) -> bool {
	let shell = &data.shell.clone();
	let mut install = match package_manager {
		"apt" => format!("apt install {}", package),
		"dnf" => format!("dnf install {}", package),
		"emerge" => format!("emerge {}", package),
		"nix-env" => format!("nix-env -iA nixpkgs.{}", package),
		"pacman" => format!("pacman -S {}", package),
		"pkg" => format!("pkg install {}", package),
		"yum" => format!("yum install {}", package),
		"zypper" => format!("zypper install {}", package),
		_ => unreachable!("Unsupported package manager"),
	};
	elevate(data, &mut install);

	let result = Command::new(shell)
		.arg("-c")
		.arg(install)
		.stdout(stderr())
		.stderr(Stdio::inherit())
		.status()
		.expect("failed to execute process");

	result.success()
}
