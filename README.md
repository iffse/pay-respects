# Pay Respects

Your command failed. Press `F` and get back to work!

- 🚀 **Fast startup**: Back to work in under 1 ms! See the
[benchmarks](#benchmarks).
- 🎯 **Checked suggestions**: No blind guesses. Suggestions are checked first!
- ✏️ **Rules in TOML**: Write fixes without touching Rust!
- 🦀 **Rust for complex cases**: Go full Rust when TOML cannot keep up!
- 🔩 **Modules in any language**: Build modules in the language you love!
- 🤖 **Optional AI**: No rule? Let AI take a shot!
- 🪶 **Small binary**: Under 1 MB. Your disk will not even notice!

![showcase](https://raw.githubusercontent.com/iffse/static-assets/refs/heads/main/pay-respects/showcase.gif)

## How to Pay Respects

Pick your shell:

<details>
	<summary>Bash / Zsh / Fish / Dash</summary>

> Add the matching line to your shell configuration file:
> ```sh
> eval "$(pay-respects bash --alias)"
> eval "$(pay-respects zsh --alias)"
> pay-respects fish --alias | source
> eval "$(pay-respects dash --alias)"
> ```
> Arguments:
> - `--alias [alias]`: Command name, defaults to `f`
> - `--shell-type <type>`: Shell type when the binary name differs
> - `--nocnf`: Disable the `command_not_found` handler
>
> The shell binary name is detected automatically. Use `--shell-type` for forks
> such as `pay-respects zsh-fork --shell-type zsh`.
>
> Dash has no inline mode or command-not-found handler.

</details>

<details>
	<summary>Xonsh</summary>

> Add this line to `~/.xonshrc`:
> ```xsh
> execx($(pay-respects xonsh --alias), 'exec', __xonsh__.ctx, filename='pay-respects')
> ```
> `Ctrl-X` `Ctrl-X` runs inline mode when prompt_toolkit is available.

</details>

<details>
	<summary>Elvish</summary>

> Add this line to `~/.config/elvish/rc.elv`:
> ```elv
> eval (pay-respects elvish --alias | slurp)
> ```
> `Ctrl-X` runs inline mode. Elvish has no command-not-found handler.

</details>

<details>
	<summary>Nushell</summary>

> Add the generated lines to your Nushell config:
> ```sh
> # replace `nu` with `nush` or `nushell` if your binary has a different name
> pay-respects nu --alias [<alias>]
> ```

> Or save them to a file and source it:
> ```sh
> pay-respects nu --alias [<alias>] | save -f ~/.pay-respects.nu
>
> source ~/.pay-respects.nu
> ```

</details>

<details>
	<summary>PowerShell</summary>

> Add the generated lines to your profile:
> ```pwsh
> # replace `pwsh` with `powershell` if your binary has a different name
> pay-respects pwsh --alias [<alias>]
> ```

> Or append them directly:
> ```pwsh
> pay-respects pwsh --alias [<alias>] >> $PROFILE
> ```

</details>

<details>
	<summary>Custom initialization for any shell</summary>

> pay-respects needs two environment variables:
>
> - `_PR_SHELL`: Binary name of the current shell
> - `_PR_LAST_COMMAND`: Last command
>
> pay-respects prints commands that the shell should evaluate, such as `cd`.
>
> Example:
> ```sh
> eval $(_PR_SHELL=zsh-fork _PR_SHELL_TYPE=zsh _PR_LAST_COMMAND="git comit" pay-respects)
> ```

> These variables are optional. They let pay-respects skip work:
>
> - `_PR_SHELL_TYPE`: Shell type. Auto-detected from `_PR_SHELL` if absent.
> - `_PR_ALIAS`: Aliases to commands, separated by newlines in zsh format,
> for example `gc=git commit`
> - `_PR_ERROR_MSG`: Error message from the previous command. If absent,
> pay-respects reads it from a multiplexer log or reruns the command.
> - `_PR_EXECUTABLES`: Space-separated list of executables. If absent,
> pay-respects scans `$PATH`.

</details>

Press **[`F` to pay respects]**.

[`F` to pay respects]: https://en.wikipedia.org/wiki/Press_F_to_pay_respects

`Ctrl+X` `Ctrl+X` runs experimental inline mode. It fixes the command in place
without executing it:

- `gitcommit` + `C-X` `C-X` → `git commit`
- `z payrespe` + `C-X` `C-X` → `cd /home/iff/Code/pay-respects`

<details>
	<summary>Integrations with other tools</summary>

> - **tmux**, **GNU Screen**, **Zellij**, **WezTerm**, **kitty**:
> 	- Read the command log from the session. This avoids rerunning the command
> to get its error message. Set `LANG=C` so the log matches reliably.
> 	- The prompt must show a prefix before the command. The generated shell
> config sets this up.
> - **zoxide**: Uses its database to navigate directories.

</details>

<details>
	<summary>Environment variable configuration</summary>

> - `_PR_LIB`: Module directories, separated like `PATH`. If unset,
> pay-respects uses `PATH` and the compile-time value.
> - `_PR_PACKAGE_MANAGER`: Package manager to use instead of guessing from an
> alphabetical list. Set it to an empty value to disable package search.
> 	- `_DEF_PR_PACKAGE_MANAGER`: Compile-time value

> `_PR_MODE` selects the run mode:
>
> - `noconfirm`: Execute a suggestion without confirmation
> - `inline`: Print the best fix without executing it
> - `echo`: Print suggestions to `stdout` without executing them
> - `cnf`: Internal mode for the command-not-found hook
>
> Example with `noconfirm`:
>
> ```sh
> # new versions
> alias ff="__pr_main noconfirm"
>
> # old versions
> function ff() {
> 	(
> 		export _PR_MODE="noconfirm"
> 		f
> 	)
> }
> ```
>
> Other options:
>
> - `_PR_NO_CONFIG`: Skip configuration files
> - `_PR_NO_DESPERATE`: Disable slow fallback checks that sometimes give a
> better result
> - `_PR_PREFIX`: Prompt prefix before the command. Needed to read logs from
> multiplexers.
> - Disable integrations:
> 	- `_PR_NO_ZOXIDE`
> 	- `_PR_NO_MULTIPLEXER`: Same as setting all of these:
> 		- `_PR_NO_TMUX`
> 		- `_PR_NO_SCREEN`
> 		- `_PR_NO_ZELLIJ`
> 		- `_PR_NO_WEZTERM`
> 		- `_PR_NO_KITTY`

</details>

## Installing

Install from your package manager if available:

[![Packaging status](https://repology.org/badge/vertical-allrepos/pay-respects.svg)](https://repology.org/project/pay-respects/versions)

<details>
	<summary>Instructions for package managers</summary>

> | OS / Distribution | Repository      | Instructions                                      |
> |-------------------|-----------------|---------------------------------------------------|
> | Arch Linux        | [AUR]           | `paru -S pay-respects` (`-bin`)                   |
> | Arch Linux (ARM)  | [Arch Linux CN] | `sudo pacman -S pay-respects`                     |
> | MacOS / *Any*     | [timescam]      | `brew install timescam/homebrew-tap/pay-respects` |
> | NixOS / *Any*     | [nixpkgs]       | `nix-env -iA nixos.pay-respects`                  |

[AUR]: https://aur.archlinux.org/
[Arch Linux CN]: https://github.com/archlinuxcn/repo
[nixpkgs]: https://github.com/NixOS/nixpkgs
[timescam]: https://github.com/timescam/homebrew-tap

</details>

Pre-built binaries are on [GitHub releases]. Install scripts are also
available. On Unix:
```sh
curl -sSfL https://raw.githubusercontent.com/iffse/pay-respects/main/install.sh | sh
```
On Windows, with PowerShell:
```pwsh
irm https://raw.githubusercontent.com/iffse/pay-respects/main/install.ps1 | iex
```

[GitHub releases]: https://github.com/iffse/pay-respects/releases

<details>
	<summary>Cargo / Compile from source (any OS and architecture supported by Rust)</summary>

> You need Cargo for this.

> Install the core package and any modules you want from
> [crates.io](https://crates.io/):
> ```sh
> cargo install pay-respects
> cargo install pay-respects-module-runtime-rules
> cargo install pay-respects-module-request-ai
> ```

> A git clone is better if you want custom compile-time rules:
> ```sh
> git clone --depth 1 https://github.com/iffse/pay-respects
> cd pay-respects
> cargo install --path core
> cargo install --path module-runtime-rules
> cargo install --path module-request-ai
> ```

</details>

## Configuration

See [configuration](./config.md).

## Rules & Modules

- [Writing rules (TOML or Rust)](./rules.md)
- [Custom modules](./modules.md)

## AI Integration

> **Disclaimer**: AI suggestions can be wrong. Check them before you accept
> one.

AI suggestions are opt-in. When `request-ai` is installed and no API key is
configured, the module asks whether to use the built-in key. You can remember
the answer in the configuration file:

```toml
[module.request-ai]
enabled = true
```

Set `enabled = false`, or set `_PR_AI_DISABLE` to any value, to turn AI
suggestions off:

```sh
export _PR_AI_DISABLE=1
```

A built-in API key ships with the source. Some distributions remove it. The
service is free while I can cover the cost. It may stop when rate limits hit.

[The proxy does not track or store
anything.](https://github.com/iffse/pay-respects-serverless)
If the service is useful to you, consider a donation:

<div>
	<a
		href="https://liberapay.com/iff/donate"
		target="_blank"
		rel="noreferrer"
		><img
			src="https://liberapay.com/assets/widgets/donate.svg"
			alt="Donate using Liberapay"
		/></a
	>
	<a href="https://ko-fi.com/iffse" target="_blank" rel="noreferrer"
		><img
			height='30'
			src="https://www.vectorlogo.zone/logos/ko-fi/ko-fi-ar21.svg"
			alt="Donate using Ko-fi"
			style="height: 30px;"
		/></a
	>
	<br />
	<a href="https://iffse.eu.org/stripe" target="_blank" rel="noreferrer"
		><img
			height='30'
			src="https://upload.wikimedia.org/wikipedia/commons/thumb/b/ba/Stripe_Logo%2C_revised_2016.svg/1280px-Stripe_Logo%2C_revised_2016.svg.png"
			alt="Donate using Stripe"
			style="height: 30px;"
		/></a
	>
	<a
		href="https://www.paypal.com/donate/?hosted_button_id=QN7Z7ZHRAAFZL"
		target="_blank"
		rel="noreferrer"
		><img
			height='30'
			src="https://upload.wikimedia.org/wikipedia/commons/b/b5/PayPal.svg"
			alt="Donate using PayPal"
			style="height: 25px; margin-bottom: 3px;"
		/></a
	>
</div>

[AI usage and API configuration](./module-request-ai/README.md)

## Contributing

The current rule format covers most cases. More rules are welcome.

See the [roadmap] for open work.

[roadmap]: ./roadmap.md

The project is mirrored on several sites. Pick the one you prefer:

- [Codeberg](https://codeberg.org/iff/pay-respects)
- [GitHub](https://github.com/iffse/pay-respects)
- [GitLab](https://gitlab.com/iffse/pay-respects)

## Licenses

- **Binaries**: AGPL-3.0
	- Core and modules
- **Libraries**: MPL-2.0
	- Parser, rule engine, utils, and select

## Benchmarks

Run `make bench` with `hyperfine` to reproduce these numbers. The results below
come from my machine (AMD Ryzen 5 5600X, DDR4 2400 MHz, HDD 7200 RPM with Btrfs
compression, glibc build). Yours will differ.

| Case                      | Results             |
|---------------------------|---------------------|
| Initialization            | 493.7 µs ±  38.1 µs |
| Privilege                 | 655.0 µs ± 100.4 µs |
| Typo: Command             | 5.1 ms ±   0.2 ms   |
| Typo: File path           | 3.6 ms ±   0.4 ms   |
| RegEx: Command            | 3.3 ms ±   0.1 ms   |
| RegEx: Options            | 3.3 ms ±   0.2 ms   |
| RegEx: Error              | 3.3 ms ±   0.1 ms   |
| Desperate: Fuzzy recovery | 703.5 µs ±  65.9 µs |
| Desperate: File look up   | 3.4 ms ±   0.2 ms   |

Caveats:

- **Integrations are off**: In real use, a delay can come from a tool such as
`zoxide`. It takes about 50 ms on its own.
- Binaries are **optimized for size, not speed**. They use regex-lite instead
of regex because it is smaller, though slower. A smaller binary loads faster,
especially on a cold run.

## Flowchart

How it works:

![flowchart](https://raw.githubusercontent.com/iffse/static-assets/refs/heads/main/pay-respects/flowchart.svg)
