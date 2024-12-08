# Changelog

All notable changes to components of this project since 0.5.14 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- `--nocnf` option in docs wasn't the same as in the code `--noncf`. They are normalized to `--nocnf`

## [0.6.0] - 2024-12-08

### Added

- Modular system
- Package manager integration for `apt` (also `snap` and `pkg` via `command-not-found`), `dnf`, `portage`, `nix`, `yum`
- Adding aliases to executable match

### Changed

- Heavy project refactoring
- `runtime-rules` and `request-ai` are now modules instead of features

## [0.5.15] - 2024-12-07

### Added

- PowerShell support by [artiga033](https://github.com/iffse/pay-respects/pull/15)
- MSYS2 fix by [mokurin000](https://github.com/iffse/pay-respects/pull/12)
- Command not found mode: Run `pay-respects` automatically by shell
	- Suggest command if a good match is found
	- If no good match is found, search if package manager (`pacman` only) has a binary with the same name and prompt to install
- Multiple suggestions

### Changed

- Major project refactoring
- Default request-AI API
- i18n updates

## [0.5.14] - 2024-11-23

History start.

[unreleased]: https://github.com/iffse/pay-respects/compare/v0.6.0..HEAD
[0.6.0]: https://github.com/iffse/pay-respects/compare/v0.5.15..v0.6.0
[0.5.15]: https://github.com/iffse/pay-respects/compare/v0.5.14..v0.5.15
[0.5.14]: https://github.com/iffse/pay-respects/commits/v0.5.14
