# Pay Respects

Typed a wrong command? Pay Respects will try to correct your wrong console command by simply pressing `F`!

- 🚀 **Blazing fast suggestion**: You won't notice any delay for asking suggestions!
- ✏️ **Easy to write rules**: You don't need to know Rust. The rules are written in a TOML file that is simple to work with and can be evaluated to Rust code upon compilation! Optional runtime user defined rules can be enabled starting from 0.5!
- 🎯 **Accurate results**: Suggestions must pass several conditions in order to be prompted to the user, no `sudo` suggestions when you are using `doas`!
- 🪶 **Tiny binary size**: Not even 1MB!

![pacman-fix](img/pacman-fix.png)

![cd-fix](img/cd-fix.png)

## How to Pay Respects

Please follow the instruction for your shell:

<details>
	<summary>Base / Zsh</summary>

Manual aliasing:
``` shell
alias f="$(pay-respects bash)"
alias f="$(pay-respects zsh)"
```

Auto aliasing (optional custom alias can be added after `--alias argument`):
``` shell
eval "$(pay-respects bash --alias)"
eval "$(pay-respects zsh --alias)"
```

</details>

<details>
	<summary>Fish</summary>

Manual aliasing:
``` shell
alias f="$(pay-respects fish)"
```

Auto aliasing (optional custom alias can be added after `--alias argument`):
``` shell
pay-respects fish --alias | source
```

</details>

<details>
	<summary>Nushell</summary>

Add the output of the following file to your configuration file:
```
pay-respects nushell [--alias <alias>]
```

Or save it as a file:
```
pay-respects nushell [--alias <alias>] | save -f ~/.pay-respects.nu
```
and source from your config file:
```
source ~/.pay-respects.nu
```

</details>

You can now **press `F` to Pay Respects**!

## Installing

Install from your package manager if available:

[![Packaging status](https://repology.org/badge/vertical-allrepos/pay-respects.svg)](https://repology.org/project/pay-respects/versions)

<details>
	<summary>Generic x86 Desktop Linux</summary>

1. Get the latest binary from [releases](https://github.com/iffse/pay-respects/releases).
```shell
curl -sL -o pay-respects.zip \
$(curl -s https://api.github.com/repos/iffse/pay-respects/releases/latest \
| sed 's/[()",{}]/ /g; s/ /\n/g' \
| grep "https.*pay-respects-ubuntu-latest.zip")
```

2. Extract zip, e.g. one of the following:
```shell
7z -x pay-respects.zip
unzip pay-respects.zip
```

3. System-wide installation:
```shell
sudo chmod a+x pay-respects
sudo mv pay-respects /usr/local/bin/pay-respects
```

4. Delete the downloaded package:
```shell
rm pay-respects.zip
```
</details>

<details>
	<summary>Compile from source (any OS/architecture)</summary>

This installation requires you to have Cargo (the Rust package manager) installed.

Install from [crates.io](https://crates.io/), `runtime-rules` is optional:
```shell
cargo install pay-respects --features=runtime-rules
```

Clone from git and install, suitable for adding custom compile-time rules:
```
git clone --depth 1 https://github.com/iffse/pay-respects
cd pay-respects
cargo install --path .
```

</details>

## Rule Files

See [writing rules](./rules.md) for how to write rules.

## Contributing

Current option to write rules should cover most of the cases.

We need more rule files, contributions are welcomed!

This project is hosted at various sites, choose the one that suits you best:

- [Codeberg](https://codeberg.org/iff/pay-respects)
- [GitHub](https://github.com/iffse/pay-respects)
- [GitLab](https://gitlab.com/iffse/pay-respects)

