# Pay Respects

Typed a wrong command or don't know what to do? Pay Respects will suggest a fix to your console command by simply pressing `F`!

- 🚀 **Blazing fast suggestion**: You won't notice any delay for asking suggestions!
- 🎯 **Accurate results**: Suggestions are verified before being prompted to the user, no `sudo` suggestions when you are using `doas`!
- ✏️ **Easy to write rules**: You don't need to know Rust. The rules are written in a TOML file!
- 🔩 **Modular**: TOML not your taste? Add sources using your favorite language with a custom module!
- 🤖 **AI Support**: AI module comes in aid when there is no rule for your error!
- 🪶 **Tiny binary size**: Not even 1MB for core features!

![showcase](./img/showcase.gif)

## How to Pay Respects

Please follow the instruction for your shell:

<details>
	<summary>Bash / Zsh / Fish</summary>

> Append the following line to your configuration file:
> ```shell
> eval "$(pay-respects bash --alias)"
> eval "$(pay-respects zsh --alias)"
> pay-respects fish --alias | source
> ```
> Arguments:
> - `--alias [alias]`: Alias to a custom key, defaults to `f`
> - `--nocnf`: Disables `command_not_found` handler

> Manual aliasing (**DEPRECATED**, do not use):
> ```shell
> alias f="$(pay-respects bash)"
> alias f="$(pay-respects zsh)"
> alias f="$(pay-respects fish)"
> ```

</details>

<details>
	<summary>Nushell</summary>

> Add the following output to your configuration file:
> ```shell
> pay-respects nushell --alias [<alias>]
> ```

> Or save it as a file:
> ```shell
> pay-respects nushell --alias [<alias>] | save -f ~/.pay-respects.nu
> ```
> and source from your config file:
> ```shell
> source ~/.pay-respects.nu
> ```

</details>

<details>
	<summary>PowerShell</summary>

> Append the following to your profile:
> ```pwsh
> pay-respects pwsh --alias [<alias>] [--nocnf] [>> $PROFILE] # use the pipe to directly append it to your profile if you like
> ```

</details>

<details>
	<summary>Custom initialization for arbitrary shell</summary>

> pay-respects only requires 2 environment variables to function:
>
> - `_PR_SHELL`: The binary name of the current working shell.
> - `_PR_LAST_COMMAND`: The last command.
>
> pay-respects echos back, if applicable, a `cd` command that can be evaluated by the current working shell.

> General example:
> ```shell
> eval $(_PR_SHELL=sh _PR_LAST_COMMAND="git comit" pay-respects)
> ```

</details>

<details>
	<summary>Environment variable configurations</summary>

> - `_PR_LIB`: Directory of modules, analogous to `PATH`. If not provided, search in `PATH` or compile-time provided value.
> - `_PR_PACKAGE_MANAGER`: Use defined package manager instead of auto-detecting alphabetically

</details>

You can now **press `F` to Pay Respects**!

## Installing

Install from your package manager if available:

[![Packaging status](https://repology.org/badge/vertical-allrepos/pay-respects.svg)](https://repology.org/project/pay-respects/versions)

<details>
	<summary>Instructions for package managers</summary>

> | OS / Distribution | Repository      | Instructions                     |
> |-------------------|-----------------|----------------------------------|
> | Arch Linux        | [AUR]           | `paru -S pay-respects` (`-bin`)  |
> | Arch Linux        | [Arch Linux CN] | `sudo pacman -S pay-respects`    |
> | NixOS / *Any*     | [nixpkgs]       | `nix-env -iA nixos.pay-respects` |

[AUR]: https://aur.archlinux.org/
[Arch Linux CN]: https://github.com/archlinuxcn/repo
[nixpkgs]: https://github.com/NixOS/nixpkgs

</details>

Alternatively, install pre-built binaries from [GitHub releases](https://github.com/iffse/pay-respects/releases). An [install script](./install.sh) is available:
```
curl -sSfL https://raw.githubusercontent.com/iffse/pay-respects/main/install.sh | sh
```

<details>
	<summary>Cargo / Compile from source (any OS/architecture supported by Rust)</summary>

> This installation requires you to have Cargo (the Rust package manager) installed.

> Install from [crates.io](https://crates.io/), modules are optional
> ```shell
> cargo install pay-respects
> cargo install pay-respects-module-runtime-rules
> cargo install pay-respects-module-request-ai
> ```

> Clone from git and install, suitable for adding custom compile-time rules:
> ```
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

> **Disclaimer**: You are using AI generated content on your own risk. Please double-check its suggestions before accepting.

AI suggestions should work out of the box with `request-ai` module installed.

An API key is included with the source. It should always work unless I can no longer afford this public service or rate limits are reached. If it's useful to you, **please share this project and spread the word**. Also consider making a donation to keep its public usage alive:

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

<details>
	<summary>AI and API Configuration</summary>

> Configuration is done via environment variables:
>
> - `_PR_AI_API_KEY`: Your own API key
> - `_PR_AI_URL`: URL used. Any OpenAI compatible URL can be used, e.g.:
>	- `https://api.openai.com/v1/chat/completions` (Note: OpenAI's ChatGPT is very slow)
>	- `https://api.groq.com/openai/v1/chat/completions`
> - `_PR_AI_MODEL`: Model used
> - `_PR_AI_DISABLE`: Setting to any value disables AI integration
> - `_PR_AI_LOCALE`: Locale in which the AI explains the suggestion. Defaults to user system locale

> Compile time variables: Default values for the respective variables above when not set
>
> - `_DEF_PR_AI_API_KEY`
> - `_DEF_PR_AI_URL`
> - `_DEF_PR_AI_MODEL`
>
>  If default values were not provided, pay-respects' own values will be used. Your request will be filtered to avoid abuse usages. Request will then be forwarded to a LLM provider that will not use your data for training.

</details>

## Contributing

Current option to write rules should cover most of the cases.

We need more rule files, contributions are welcomed!

This project is hosted at various sites, choose the one that suits you best:

- [Codeberg](https://codeberg.org/iff/pay-respects)
- [GitHub](https://github.com/iffse/pay-respects)
- [GitLab](https://gitlab.com/iffse/pay-respects)

