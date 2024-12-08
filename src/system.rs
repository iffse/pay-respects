use crate::shell::{command_output, elevate, Data};
use std::io::stderr;
use std::process::Command;
use std::process::Stdio;
use colored::Colorize;

pub fn get_package_manager(data: &mut Data) -> Option<String> {
	let package_managers = vec![
		"apt", "dnf", "emerge", "nix", "pacman",
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

pub fn get_packages(
	data: &mut Data,
	package_manager: &str,
	executable: &str,
) -> Option<Vec<String>> {
	let shell = &data.shell.clone();
	match package_manager {
		"apt" => {
			if !data.has_executable("apt-file") {
				eprintln!("{}: apt-file is required to find packages", "pay-respects".yellow());
				return None;
			}
			let result = command_output(
				shell,
				&format!("apt-file find --regexp '.*/bin/{}$'", executable),
			);
			if result.is_empty() {
				return None;
			}
			let packages: Vec<String> = result
				.lines()
				.map(|line| line.split_once(':').unwrap().0.to_string())
				.collect();

			if packages.is_empty() {
				None
			} else {
				Some(packages)
			}
		}
		"dnf" => {
			let result = command_output(shell, &format!("dnf provides {}", executable));
			if result.is_empty() {
				return None;
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
		"emerge" => {
			if !data.has_executable("e-file") {
				eprintln!("{}: pfl is required to find packages", "pay-respects".yellow());
				return None;
			}
			let result = command_output(shell, &format!("e-file /usr/bin/{}", executable));
			if result.is_empty() {
				return None;
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
		}
		"nix" => {
			if !data.has_executable("nix-locate") {
				eprintln!("{}: nix-index is required to find packages", "pay-respects".yellow());
				return None;
			}
			let result = command_output(shell, &format!("nix-locate 'bin/{}'", executable));
			if result.is_empty() {
				return None;
			}
			let packages: Vec<String> = result
				.lines()
				.map(|line| {
					line.split_whitespace()
						.next()
						.unwrap()
						.rsplit_once('.')
						.unwrap()
						.0
						.to_string()
				})
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
			if result.is_empty() {
				return None;
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
		_ => {
			match package_manager.ends_with("command-not-found") {
				true => {
					let result = command_output(shell, &format!("command-not-found {}", executable));
					if result.is_empty() {
						return None;
					}
					if result.contains("did you mean") || result.contains("is not installed") {
						let packages = result
							.lines()
							.skip(1)
							.map(|line| line.to_string())
							.collect();
						return Some(packages);
					}
					return None;
				},
				false => unreachable!("Unsupported package manager"),
			}
		}
	}
}

pub fn install_package(data: &mut Data, package_manager: &str, package: &str) -> bool {
	let shell = &data.shell.clone();
	let mut install = match package_manager {
		"apt" => format!("apt install {}", package),
		"dnf" => format!("dnf install {}", package),
		"emerge" => format!("emerge {}", package),
		"nix" => format!("nix profile install nixpkgs#{}", package),
		"pacman" => format!("pacman -S {}", package),
		"pkg" => format!("pkg install {}", package),
		"yum" => format!("yum install {}", package),
		"zypper" => format!("zypper install {}", package),
		_ => {
			match package_manager.ends_with("command-not-found") {
				true => {
					match package.starts_with("Command") {
						false => package.to_string(),
						true => {
							let split = package.split_whitespace().collect::<Vec<&str>>();
							let command = split[1];
							let package = split[split.len() - 1];
							let new_command = data.command.clone().replacen(command, package, 1);
							data.update_command(&new_command);
							format!("apt install {}", package)
						}
					}
				},
				false => unreachable!("Unsupported package manager"),
			}
		},
	};
	elevate(data, &mut install);

	#[cfg(debug_assertions)]
	eprintln!("install: {}", install);

	let result = Command::new(shell)
		.arg("-c")
		.arg(install)
		.stdout(stderr())
		.stderr(Stdio::inherit())
		.status()
		.expect("failed to execute process");

	result.success()
}
