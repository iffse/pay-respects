use crate::data::Data;
use crate::shell::command_output_or_error;
use crate::shell::{command_output, elevate};
use crate::style::{print_warning, unexpected_format};
use std::io::stderr;
use std::process::Command;
use std::process::Stdio;

use crate::config::InstallMethod;

pub fn get_package_manager(data: &mut Data) -> Option<String> {
	if let Ok(package_manager) = std::env::var("_PR_PACKAGE_MANAGER") {
		if package_manager.is_empty() {
			return None;
		}
		return Some(package_manager);
	}

	if let Some(package_manager) = data.config.package_manager.as_ref() {
		if package_manager.is_empty() {
			return None;
		}
		return Some(package_manager.to_string());
	}

	if let Some(package_manager) = option_env!("_DEF_PR_PACKAGE_MANAGER") {
		if package_manager.is_empty() {
			return None;
		}
		return Some(package_manager.to_string());
	}

	for package_manager in &[
		"apt", "dnf", "emerge", "guix", "nix", "pacman", "yum",
		// "zypper",
	] {
		if data.executables.iter().any(|exe| exe == package_manager) {
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
				print_warning("apt-file is required to find packages");
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
				.map(|line| {
					let bin = line.split_once(':');
					if let Some((pkg, _)) = bin {
						pkg.to_string()
					} else {
						unexpected_format(line);
						"".to_string()
					}
				})
				.filter(|s| !s.is_empty())
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
				print_warning("pfl is required to find packages");
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
		"guix" => {
			let result =
				command_output(shell, &format!("{} locate {}", package_manager, executable));
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
		"nix" => {
			let packages: Vec<String>;
			if data.executables.contains(&"nix-locate".to_string()) {
				let result =
					command_output(shell, &format!("nix-locate --regex 'bin/{}$'", executable));
				if result.is_empty() {
					return None;
				}
				packages = result
					.lines()
					.map(|line| {
						let out = line.split_whitespace().next().unwrap().rsplit_once('.');

						if let Some((pkg, _)) = out {
							pkg.to_string()
						} else {
							unexpected_format(line);
							"".to_string()
						}
					})
					.filter(|s| !s.is_empty())
					.collect();
			} else if data.executables.contains(&"nix-search".to_string()) {
				let result = command_output(shell, &format!("nix-search '{}'", executable));
				if result.is_empty() {
					return None;
				}
				packages = result
					.lines()
					.map(|line| line.split_whitespace().next().unwrap().to_string())
					.collect()
			} else {
				print_warning("nix-locate or nix-search is required to find packages");
				return None;
			};

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
				let result =
					command_output_or_error(shell, &format!("{} {}", package_manager, executable));
				if result.is_empty() {
					return None;
				}
				if result.contains("did you mean")
					|| result.contains("is not installed")
					|| result.contains("can be installed")
				{
					let packages = result
						.lines()
						.skip(1)
						.map(|line| line.trim().to_string())
						.collect();
					return Some(packages);
				}
				None
			}
			false => {
				print_warning("Unsupported package manager");
				None
			}
		},
	}
}

pub fn install_string(data: &mut Data, package_manager: &str, package: &str) -> String {
	let method = &data.config.install_method;
	match package_manager {
		"apt" | "dnf" | "pkg" | "yum" | "zypper" => {
			format!("{} install {}", package_manager, package)
		}
		"emerge" => format!("emerge {}", package),
		"guix" => {
			if method == &InstallMethod::Shell {
				return format!("guix shell {}", package);
			}
			format!("guix package -i {}", package)
		}
		"nix" => {
			if method == &InstallMethod::Shell {
				return format!("nix-shell -p {}", package,);
			}
			format!("nix profile install nixpkgs#{}", package)
		}
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
	}
}

pub fn install_package(data: &mut Data, package_manager: &str, install: &str) -> bool {
	let shell = data.shell.clone();
	let mut install = install.to_string();
	// guix and nix do not require privilege escalation
	#[allow(clippy::single_match)]
	match package_manager {
		"guix" | "nix" => {}
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

pub fn shell_package(data: &Data, package_manager: &str, install: &str) -> String {
	let command = data.command.clone();

	match package_manager {
		"guix" => format!("{} -- {}", install, command),
		"nix" => format!(r#"{} --command "{};return""#, install, command),
		_ => unreachable!("Only `nix` and `guix` are supported for shell installation"),
	}
}
