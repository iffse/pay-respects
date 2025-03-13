# Changelog

All notable changes to components of this project since 0.5.14 will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Nushell: Added alias support
	- Also allows arbitrary shell to provide support

### Fixed

- No longer having newlines when expanding alias

### Changed

- (Windows) Separator for `_PR_LIB` has changed to `;` by [codyduong](https://github.com/iffse/pay-respects/pull/37)

## [0.6.13] - 2025-02-12

### Changed

- CI binaries now use statically linked musl library
- Multi-suggest format changed to unordered bullet list
- Single suggests merged into multi-suggest

## [0.6.12] - 2025-01-26

### Fixed

- `nix-index` panic by [jakobhellermann](https://github.com/iffse/pay-respects/pull/31)

### Changed

- Executables environment variable passed to modules is now limited to 100k characters
- Changed the format for multi-suggest

## [0.6.11] - 2025-01-18

### Fixed

- No longer panics when interrupting multi-suggest
- Bash & Zsh: Reverted function based initialization to alias

## [0.6.10] - 2025-01-07

### Fixed

- Wrong starting distance when including all candidates
- Spacings for `opt` placeholder

### Changed

- Merged `exes` placeholder of last version into new `select` placeholder

## [0.6.9] - 2025-01-06

### Added

- Include all candidates with the same distances for executable typos

### Changed

- Running standard modules in a separated thread
- Bash init: use `fc` instead of history

## [0.6.8] - 2025-01-02

### Fixed

- Broken rule for `git` in the last version

### Removed

- Removed binary files from history. Hash of all relevant commits will change

## [0.6.7] - 2024-12-31

### Fixed

- No longer running `get_error` in CNF mode (makes PowerShell hang with recursive calls)
- Not showing `sudo` in successive suggestions (although they were applied)

### Changed

- Licenses for libraries changed to MPL-2.0 from AGPL-3.0

## [0.6.6] - 2024-12-18

### Added

- RPM packaging

### Fixed

- Panic on `sudo` input command

## [0.6.5] - 2024-12-13

### Added

- AI module: Show raw body on parse failure (sometime the AI forgets a bracket)

### Fixed

- Not getting `stderr` from command-not-found

## [0.6.4] - 2024-12-12

### Added

- Flakes install in `nix`
- Override package manager using `_PR_PACKAGE_MANAGER`
- AI module:
	- Allow multiple suggestions
	- More default values

### Changed

- Compile-time `_PR_LIB` changed to `_DEF_PR_LIB` to be explicit

## [0.6.3] - 2024-12-11

### Added

- FHS 3.0 compliance: Compile-time and runtime environment variable `_PR_LIB` specifying `lib` directories for storing modules, separated by `:`
	- Search in `PATH` if not provided

## [0.6.2] - 2024-12-10

### Added

- Aliases matching to command-not-found
- Relative path command fixes
	- Does not work in `bash` and `zsh`: Not considered a command

### Changed

- **BREAKING:** Executable list passed to modules is now a space ` ` instead of a comma `,`
- Skip privilege elevation for `nix`

## [0.6.1] - 2024-12-09

### Added

- Custom priority for modules

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

[unreleased]: https://github.com/iffse/pay-respects/compare/v0.6.13..HEAD
[0.6.13]: https://github.com/iffse/pay-respects/compare/v0.6.12..v0.6.13
[0.6.12]: https://github.com/iffse/pay-respects/compare/v0.6.11..v0.6.12
[0.6.11]: https://github.com/iffse/pay-respects/compare/v0.6.10..v0.6.11
[0.6.10]: https://github.com/iffse/pay-respects/compare/v0.6.9..v0.6.10
[0.6.9]: https://github.com/iffse/pay-respects/compare/v0.6.8..v0.6.9
[0.6.8]: https://github.com/iffse/pay-respects/compare/v0.6.7..v0.6.8
[0.6.7]: https://github.com/iffse/pay-respects/compare/v0.6.6..v0.6.7
[0.6.6]: https://github.com/iffse/pay-respects/compare/v0.6.5..v0.6.6
[0.6.5]: https://github.com/iffse/pay-respects/compare/v0.6.4..v0.6.5
[0.6.4]: https://github.com/iffse/pay-respects/compare/v0.6.3..v0.6.4
[0.6.3]: https://github.com/iffse/pay-respects/compare/v0.6.2..v0.6.3
[0.6.2]: https://github.com/iffse/pay-respects/compare/v0.6.1..v0.6.2
[0.6.1]: https://github.com/iffse/pay-respects/compare/v0.6.0..v0.6.1
[0.6.0]: https://github.com/iffse/pay-respects/compare/v0.5.15..v0.6.0
[0.5.15]: https://github.com/iffse/pay-respects/compare/v0.5.14..v0.5.15
[0.5.14]: https://github.com/iffse/pay-respects/commits/v0.5.14
