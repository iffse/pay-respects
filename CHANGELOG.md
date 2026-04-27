# Changelog

All notable changes to components of this project since 0.5.14 will be
documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Semantic Versioning].

[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/
[Semantic Versioning]:  https://semver.org/spec/v2.0.0.html

## [Unreleased]

## [0.8.8]

### Fixed

- Bash and Zsh: Command not found handler was not getting the arguments after
reworked templates
- AI module: Removed `extra_body` from requests if unused.

### Added

- Selection now supports pagination.
- AI module: Added `extra` field.

## [0.8.7]

This release fixes various problems specific to Windows.

### Breaking Changes

- For Nushell and PowerShell, the shell argument will now be treated as the
shell executable name.
 - If you were using `pay-respects nushell` and your binary is named `nu`, you
	 have to change it to `pay-respects nu`

### Fixed

- Windows:
	- Not actually stripping extensions
	- Not getting non-localized outputs
	- Draining terminal inputs
- Recovered commits that were supposed to be included in the last release

### Added

- AI module: `extra_body` field for model customization

## [0.8.6]

### Added

- Added `run0` and`sudo-rs` to internal privilege elevation list
- Alias expansion for PowerShell
- Accepted suggestions are now appended to the shell history on Nushell
and PowerShell, matching the existing behavior for Bash, Zsh, and Fish
(codeberg #36).

### Changed

- All initialization templates have been reworked

### Fixed

- Fallback modules not being called
- More robust tag parsing for AI module

## [0.8.5] - 2026-04-07

### Added

- **Automatic shell prefix detection**, no longer requires manually setting
`_PR_PREFIX` for multiplexer supports

### Fixed

- Wrong indentation and wrong capture when using multiplexer integrations

## [0.8.4] - 2026-04-06

### Added

- Inline mode rule support for runtime rules

### Changed

- **Terminal multiplexer integration now requires configuration**
- Desperate functions are now only executed when no previous suggestion is
found, improving performance

### Fixed

- Compile time parser now supports quotes in patterns

## [0.8.3] - 2026-04-05

### Added

- **GNU Screen**, **Zellij**, **WezTerm**, and **kitty** integrations
- Short command fixes: `gi tpush` can now be fixed into `git push`.
- Fuzzy recovery now provides support for options. Instead of `ls
--group-directories-first --color`, was suggesting `ls group directory first
color`

### Changed

- Logic for CNF is restored to the one used during older versions, as the
aggressive fuzzy search created too many false positives. E.g. instead of
installing `fastfetch`, was suggesting `last fetch`
- Difference highlighting no longer highlights comments.

### Fixed

- `err` placeholder changes during [0.7.13] wasn't adapted to all codebase,
such that error messages obtained through retries are still in lower cases.
- `tmux` integration wasn't complete like the point above, which wasn't being
used when retrying.
- The introduction of Inline mode on [0.8.1] come with a bug that will rerun
the previous command even on Inline and CNF modes. Shouldn't be a big problem
most of the time, excepting performance penalty.
- Comments are no longer part of the command to be fixed. However, modules will
still get the command with the comments.

## [0.8.2] - 2026-04-04

### Added

- **`tmux` integration**: No longer needs to rerun your command if you are inside
a tmux session with English locale.
- **Inline fixes**: Fixing commands on the fly, with no execution:
	- All supported shell now have a new key-binding on `Ctrl-X`
	- Fixes current typed command as good as possible:
		- `gitcommit` + `C-X` `C-X` → `git commit`
		- `z payrespe` + `C-X` `C-X` → `cd /home/iff/Code/pay-respects`
- Better search algorithm including: Segmentation, fuzzy search, etc.

## [0.8.1] - 2026-04-03

### Added

- **`zoxide` integration**: Usable for both `cd` and `z` fixes, when `zoxide`
is installed.
- **Rust rules**: Now rules can be written in Rust natively for complex logics
- **Trigram search**: New default search algorithm, with higher precision
- **Suggestions for blocking commands**: Now if a command is in a given list of
blocking commands, it won't try to run (e.g. interactive commands that do not
return anything). This allows fixing commands such as `vim file-with-typo`
- `extends` field for rule files: Allows reusing existing rules of other files
- `merge_commands` field for configuration files: Make multiple commands to use
the same rules

## [0.8.0] - 2026-04-02

### Changed

- Introducing an in-house selection UI! Now we have a UI of a great taste (which was not possible with an external library)!
	- Discrete highlighting between selected and non-selected suggestions
	- Quick selection with shortcuts (keys from 1 to 9)

## [0.7.13] - 2026-04-02

### Changed

- **Reproducibility**: Every single version before is not actually
	*reproducible* because the procedural generation of Rust code places codes in
	a random order. This does not affect how the program performs, but does make
	the hash and binary sizes differs between different builds.
	- [Unreproducible tests from
		Debian.](https://tests.reproducible-builds.org/debian/rb-pkg/unstable/amd64/pay-respects.html)
	- Source code is now reproducible, and technically should be for binaries as
		well.
- `err` placeholder is now case-sensitive to preserve usable upper-cases for
	suggestions.

### Added

- Configurable linguistic distances

### Fixed

- Nushell: Paths are now correctly formatted (`\ ` is unsupported by Nushell)

## [0.7.12] - 2026-02-08

### Fixed

- Default timeout was 3 milliseconds instead of 3 seconds, therefore last
version is not usable unless having a configuration file.
- Rolling back `reqwest` to version `0.12`. New version does not work on
android due to `rustls-platform-verifier`, despite the `0.13.1` changelog says
something is fixed.

## [0.7.11] - 2026-02-07 [YANKED]

### Added

- Layered configuration, allowing a system-wide configuration

### Fixed

- HTTPS connection of AI module

### Changed

- Default AI proxy is now serverless (URL changed)

## [0.7.10] - 2026-02-03

Maintenance release, no major changes.

### Fixed

- Many panics in various rare cases

### Added

- New rules for `cargo`, `snap`, `git`, and `jj`

## [0.7.9] - 2025-08-26

### Changed

- Install command is now explicit
- Nix and Guix: Default installation for missing commands changed to shell
- Nix: Adding `nix-search` as package query tool in addition to `nix-index`

## [0.7.8] - 2025-06-12

### Fixed

- Nix: Shell install via `nix-shell`

## [0.7.7] - 2025-06-11

### Added

- Configuration file, allowing to customize some parameters
	- (Nix/Guix): Installation method as shell, without installing to system
	profile
	- Option for let the working shell be responsible for the suggestions

### Changed

- Terminal environment variable for locales now has higher priority than system
locales (MacOS or Windows that have different locales between system and `LANG`)

### Fixed

- MacOS: Fixed arguments not available in MacOS


## [0.7.6] - 2025-04-22

### Added

- Compile-time variable to specify package manager (to be set by each
distribution)
- Rules for `size` and `brew`
- General rule parsing for `runtime-rules`

### Changed

- Re-enabled filtering when selecting candidate (`jk` does not work as Vim mode
is also enabled)
	- Workaround to move terminal cursor away from last line (cannot hide as
	`inquire` controls the cursor)

### Fixed

- Fixed panics for commands starting with a character more than 1 byte
- Fish: Don't run CNF mode recursively (in case that user's config does not
have an early return in non-interactive session)

## [0.7.5] - 2025-04-10

### Fixed

- Multi-line suggestions are run multiple times in the last release instead of
adding to history

## [0.7.4] - 2025-04-10 [YANKED]

### Added

- Adding executed commands to history for Bash, Zsh, and Fish

### Fixed

- PowerShell's init wasn't executing returned commands to be evaluated

## [0.7.3] - 2025-04-09

### Added

- Regex support for conditions matching
	- `,` cannot be used though

### Changed

- Using Damerau variation for string comparison

### Fixed

- Panics in core and runtime-rules module
- Removed duplicated characters in stream output

### Removed

- `exe_contains` rule as it can be done with regex

## [0.7.2] - 2025-04-08

### Added

- Streaming output support for AI module
	- Wasn't easy as my brain is pretty much dead at the time of writing
- `guix` support in package installation by [gs-101](https://github.com/iffse/pay-respects/pull/44)

### Fixed

- Redundant packages from `nix-index` by [SigmaSquadron](https://github.com/iffse/pay-respects/pull/45)

### Removed

- No longer depends on `libcurl`. Now using `rustls`

## [0.7.1] - 2025-04-06

### Added

- Support reasoning AI models (can take more than 20 seconds)
- Allow adding additional prompts for role-playing with perversion or whatever
- `exe_contains` condition to check if the command contains the argument

### Fixed

- Parsing command environment variables (e.g. `LANG=ja_JP.UTF-8 pacman` will
work as intended)
- Not getting `command-not-found`'s output as it goes into `stderr`

## [0.7.0] - 2025-04-05

### Breaking

- Manual aliasing no longer supported

### Added

- `noconfirm` mode: Run suggestions without confirmation
- Suggestion tests

### Fixed

- PowerShell's initialization for versions that does not support `Get-Error`

### Changed

- Reimplemented initialization with templates

## [0.6.14] - 2025-03-13

### Added

- Nushell: Added alias support
	- Also allows arbitrary shell to provide support
- `echo` mode: Only print suggestion

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

- Executables environment variable passed to modules is now limited to 100k
characters
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

- No longer running `get_error` in CNF mode (makes PowerShell hang with
recursive calls)
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

- FHS 3.0 compliance: Compile-time and runtime environment variable `_PR_LIB`
specifying `lib` directories for storing modules, separated by `:`
	- Search in `PATH` if not provided

## [0.6.2] - 2024-12-10

### Added

- Aliases matching to command-not-found
- Relative path command fixes
	- Does not work in `bash` and `zsh`: Not considered a command

### Changed

- **BREAKING:** Executable list passed to modules is now a space ` ` instead of
a comma `,`
- Skip privilege elevation for `nix`

## [0.6.1] - 2024-12-09

### Added

- Custom priority for modules

### Changed

- `--nocnf` option in docs wasn't the same as in the code `--noncf`. They are
normalized to `--nocnf`

## [0.6.0] - 2024-12-08

### Added

- Modular system
- Package manager integration for `apt` (also `snap` and `pkg` via
`command-not-found`), `dnf`, `portage`, `nix`, `yum`
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
	- If no good match is found, search if package manager (`pacman` only) has a
	binary with the same name and prompt to install
- Multiple suggestions

### Changed

- Major project refactoring
- Default request-AI API
- i18n updates

## [0.5.14] - 2024-11-23

History start.

[unreleased]: https://github.com/iffse/pay-respects/compare/v0.8.8..HEAD
[0.8.8]: https://github.com/iffse/pay-respects/compare/v0.8.7..v0.8.8
[0.8.7]: https://github.com/iffse/pay-respects/compare/v0.8.6..v0.8.7
[0.8.6]: https://github.com/iffse/pay-respects/compare/v0.8.5..v0.8.6
[0.8.5]: https://github.com/iffse/pay-respects/compare/v0.8.4..v0.8.5
[0.8.4]: https://github.com/iffse/pay-respects/compare/v0.8.3..v0.8.4
[0.8.3]: https://github.com/iffse/pay-respects/compare/v0.8.2..v0.8.3
[0.8.2]: https://github.com/iffse/pay-respects/compare/v0.8.1..v0.8.2
[0.8.1]: https://github.com/iffse/pay-respects/compare/v0.8.0..v0.8.1
[0.8.0]: https://github.com/iffse/pay-respects/compare/v0.7.13..v0.8.0
[0.7.13]: https://github.com/iffse/pay-respects/compare/v0.7.12..v0.7.13
[0.7.12]: https://github.com/iffse/pay-respects/compare/v0.7.11..v0.7.12
[0.7.11]: https://github.com/iffse/pay-respects/compare/v0.7.10..v0.7.11
[0.7.10]: https://github.com/iffse/pay-respects/compare/v0.7.9..v0.7.10
[0.7.9]: https://github.com/iffse/pay-respects/compare/v0.7.8..v0.7.9
[0.7.8]: https://github.com/iffse/pay-respects/compare/v0.7.7..v0.7.8
[0.7.7]: https://github.com/iffse/pay-respects/compare/v0.7.6..v0.7.7
[0.7.6]: https://github.com/iffse/pay-respects/compare/v0.7.5..v0.7.6
[0.7.5]: https://github.com/iffse/pay-respects/compare/v0.7.4..v0.7.5
[0.7.4]: https://github.com/iffse/pay-respects/compare/v0.7.3..v0.7.4
[0.7.3]: https://github.com/iffse/pay-respects/compare/v0.7.2..v0.7.3
[0.7.2]: https://github.com/iffse/pay-respects/compare/v0.7.1..v0.7.2
[0.7.1]: https://github.com/iffse/pay-respects/compare/v0.7.0..v0.7.1
[0.7.0]: https://github.com/iffse/pay-respects/compare/v0.6.14..v0.7.0
[0.6.14]: https://github.com/iffse/pay-respects/compare/v0.6.13..v0.6.14
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
