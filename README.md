# Pay Respects

Typed a wrong command or don't know what to do? Pay Respects will suggest a fix to your console command by simply pressing `F`!

- 🚀 **Blazing fast suggestion**: You won't notice any delay for asking
suggestions!
- 🎯 **Accurate results**: Suggestions are verified before being prompted to
the user, no `sudo` suggestions when you are using `doas`!
- ✏️ **Easy to write rules**: You don't need to know Rust. The rules are
written in a TOML file!
- 🔩 **Modular**: TOML not your taste? Add sources using your favorite language
with a custom module!
- 🤖 **AI Support**: AI module comes in aid when there is no rule for your
error!
- 🪶 **Tiny binary size**: Not even 1MB for core features!

![showcase](https://raw.githubusercontent.com/iffse/static-assets/refs/heads/main/pay-respects/showcase.gif)

## How to Pay Respects

Please follow the instruction for your shell:

<details>
	<summary>Bash / Zsh / Fish</summary>

> Append the following line to your configuration file (`--alias` no longer
> required for v0.7+):
> ```sh
> eval "$(pay-respects bash --alias)"
> eval "$(pay-respects zsh --alias)"
> pay-respects fish --alias | source
> ```
> Arguments:
> - `--alias [alias]`: Alias to a custom key, defaults to `f`
> - `--nocnf`: Disables `command_not_found` handler

> Manual aliasing (**REMOVED** after v0.7):
> ```sh
> alias f="$(pay-respects bash)"
> alias f="$(pay-respects zsh)"
> alias f="$(pay-respects fish)"
> ```

</details>

<details>
	<summary>Nushell</summary>

> Add the following output to your configuration file:
> ```sh
> pay-respects nushell --alias [<alias>]
> ```

> Or save it as a file:
> ```sh
> pay-respects nushell --alias [<alias>] | save -f ~/.pay-respects.nu
> ```
> and source from your config file:
> ```sh
> source ~/.pay-respects.nu
> ```

</details>

<details>
	<summary>PowerShell</summary>

> Append the following output to your profile:
> ```pwsh
> pay-respects pwsh --alias [<alias>]
> ```

> Or directly pipe the output to your profile:
> ```pwsh
> pay-respects pwsh --alias [<alias>] >> $PROFILE
> ```

</details>

<details>
	<summary>Custom initialization for arbitrary shell</summary>

> pay-respects only requires 2 environment variables to function:
>
> - `_PR_SHELL`: The binary name of the current working shell
> - `_PR_LAST_COMMAND`: The last command
>
> pay-respects echos back, if applicable, a `cd` command that can be evaluated
> by the current working shell.

> General example:
> ```sh
> eval $(_PR_SHELL=sh _PR_LAST_COMMAND="git comit" pay-respects)
> ```

> Following variables are not required, but can be used to reduce unnecessary
> operations:
>
> - `_PR_ALIAS`: A list of aliases to commands. Separated by newlines with
> zsh-like formatting, e.g. `gc=git commit`
> - `_PR_ERROR_MSG`: Error message from the previous command. `pay-respects`
> will rerun previous command to get the error message if absent
> - `_PR_EXECUTABLES`: A space separated list of commands/executables.
> `pay-respects` will search for `$PATH` if absent

</details>

<details>
	<summary>Environment variable configurations</summary>

> - `_PR_LIB`: Directory of modules, analogous to `PATH`. If not provided,
>   search in `PATH` or compile-time provided value
> - `_PR_PACKAGE_MANAGER`: Use defined package manager instead of
> auto-detecting alphabetically. Empty value disables package search
> functionality
> 	- `_DEF_PR_PACKAGE_MANAGER`: compile-time value

> You can specify different modes to run with `_PR_MODE`:
>
> - `noconfirm`: Execute suggestions without confirm
> - `echo`: Print suggestions to `stdout` without executing
> - `cnf`: Used for command not found hook
>
> Example usage with `noconfirm`:
>
> ```sh
> function ff() {
> 	(
> 		export _PR_MODE="noconfirm"
> 		f
> 	)
> }
> ```

</details>

You can now **[Press `F` to Pay Respects]**!

[Press `F` to Pay Respects]: https://en.wikipedia.org/wiki/Press_F_to_pay_respects

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

Alternatively, install pre-built binaries from [GitHub releases]. An [install
script] is available:
```sh
curl -sSfL https://raw.githubusercontent.com/iffse/pay-respects/main/install.sh | sh
```
[GitHub releases]: https://github.com/iffse/pay-respects/releases
[install script]: ./install.sh

<details>
	<summary>Cargo / Compile from source (any OS/architecture supported by Rust)</summary>

> This installation requires you to have Cargo (the Rust package manager) installed.

> Install from [crates.io](https://crates.io/), modules are optional
> ```sh
> cargo install pay-respects
> cargo install pay-respects-module-runtime-rules
> cargo install pay-respects-module-request-ai
> ```

> Clone from git and install, suitable for adding custom compile-time rules:
> ```sh
> git clone --depth 1 https://github.com/iffse/pay-respects
> cd pay-respects
> cargo install --path core
> cargo install --path module-runtime-rules
> cargo install --path module-request-ai
> ```

</details>

## Rules & Modules

See the following pages:

- [Writing rules (TOML)](./rules.md)
- [Custom modules](./modules.md)

## AI Integration

> **Disclaimer**: You are using AI generated content on your own risk. Please
> double-check its suggestions before accepting.

AI suggestions should work out of the box with `request-ai` module installed.

An API key is included with the source (your distribution might have stripped
them out). It should always work unless I can no longer afford this public
service or rate limits are reached. If it's useful to you, consider making a
donation:

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
			src="https://cdn.brandfolder.io/KGT2DTA4/at/8vbr8k4mr5xjwk4hxq4t9vs/Stripe_wordmark_-_blurple.svg"
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

[AI usages and API configurations](./module-request-ai/README.md)

## Contributing

Current option to write rules should cover most of the cases. We need more
rules, contributions are welcomed!

There's also a [roadmap] for contribution opportunities.

[roadmap]: ./roadmap.md

This project is hosted at various sites, you can choose the one that you feel
most comfortable with:

- [Codeberg](https://codeberg.org/iff/pay-respects)
- [GitHub](https://github.com/iffse/pay-respects)
- [GitLab](https://gitlab.com/iffse/pay-respects)

## Licenses

- **Binaries**: AGPL-3.0
	- Core and modules
- **Libraries**: MPL-2.0
	- Parser and utils
