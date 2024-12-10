use crate::shell::{command_output, elevate, Data};
use colored::Colorize;
use std::io::stderr;
use std::process::Command;
use std::process::Stdio;

pub fn get_package_manager(data: &mut Data) -> Option<String> {
	let package_managers = vec![
		"apt", "dnf", "emerge", "nix", "pacman", "yum",
		// "zypper",
	];

	for package_manager in package_managers {
		if data.executables.contains(&package_manager.to_string()) {
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
			if !data.executables.contains(&"apt-file".to_string()) {
				eprintln!(
					"{}: apt-file is required to find packages",
					"pay-respects".yellow()
				);
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
		"dnf" | "yum" => {
			let result = command_output(
				shell,
				&format!("{} provides '/usr/bin/{}'", package_manager, executable),
			);
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
			if !data.executables.contains(&"e-file".to_string()) {
				eprintln!(
					"{}: pfl is required to find packages",
					"pay-respects".yellow()
				);
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
			if !data.executables.contains(&"nix-locate".to_string()) {
				eprintln!(
					"{}: nix-index is required to find packages",
					"pay-respects".yellow()
				);
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
			let result = if data.executables.contains(&"pkgfile".to_string()) {
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
		_ => match package_manager.ends_with("command-not-found") {
			true => {
				let result = command_output(shell, &format!("{} {}", package_manager, executable));
				if result.is_empty() {
					return None;
				}
				if result.contains("did you mean") || result.contains("is not installed") {
					let packages = result
						.lines()
						.skip(1)
						.map(|line| line.trim().to_string())
						.collect();
					return Some(packages);
				}
				None
			}
			false => unreachable!("Unsupported package manager"),
		},
	}
}

pub fn install_package(data: &mut Data, package_manager: &str, package: &str) -> bool {
	let shell = &data.shell.clone();
	let mut install = match package_manager {
		"apt" | "dnf" | "pkg" | "yum" | "zypper" => {
			format!("{} install {}", package_manager, package)
		}
		"emerge" => format!("emerge {}", package),
		"nix" => format!("nix profile install nixpkgs#{}", package),
		"pacman" => format!("pacman -S {}", package),
		_ => match package_manager.ends_with("command-not-found") {
			true => match package.starts_with("Command ") {
				false => package.to_string(),
				true => {
					let split = package.split_whitespace().collect::<Vec<&str>>();
					let command = split[1];
					let package = split[split.len() - 1];
					let new_command = data.command.clone().replacen(&data.split[0], command, 1);
					data.update_command(&new_command);
					format!("apt install {}", package)
				}
			},
			false => unreachable!("Unsupported package manager"),
		},
	};

	// nix does not require privilege escalation
	#[allow(clippy::single_match)]
	match package_manager {
		"nix" => {}
		_ => elevate(data, &mut install),
	}

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
