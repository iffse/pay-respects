# Pay Respects

Typed a wrong command? Pay Respects will try to correct your wrong console command by simply pressing `F`!

- 🚀 **Blazing fast suggestion**: You won't notice any delay for asking suggestions!
- ✏️ **Easy to write rules**: You don't need to know Rust. The rules are written in a TOML file that is simple to work with and can be evaluated to Rust code upon compilation! Optional runtime user defined rules can be enabled starting from 0.5!
- 🎯 **Accurate results**: Suggestions must pass several conditions in order to be prompted to the user, no `sudo` suggestions when you are using `doas`!
- 🪶 **Tiny binary size**: Not even 1MB!

![pacman-fix](img/pacman-fix.png)

![cd-fix](img/cd-fix.png)

## How to Pay Respects

The binary is named `pay-respects`, by adding an alias to your shell
configuration:
``` shell
# Note: You may need to have the binary exposed in your path
alias f="$(pay-respects <your_shell_here>)"

# for example, using `zsh`:
alias f="$(pay-respects zsh)"

# Alternatively, you can also use the following initialization in your config file
# for bash and zsh
eval "$(pay-respects <shell> --alias)"
# for fish
pay-respects fish --alias | source

# for `nushell`, the alias can be added automatically with:
pay-respects nushell
```
You can now **press `F` to Pay Respects**!

Currently, only corrections to `bash`, `zsh`, and `fish` are working flawlessly.

`nushell` is currently usable, but there is no alias expansion, and you will have to put the evaluated initialization command in your config file (added automatically with `pay-respects nushell`). In addition, commands that need to be evaluated in the current working shell (such as `cd`) cannot yet be implemented in `nushell`.

Shell is not obtained automatically through environment variables because it won't work with nested shells, e.g. `fish` inside `zsh` still has `SHELL=zsh`.

## Installing

Install from your package manager if available:

[![Packaging status](https://repology.org/badge/vertical-allrepos/pay-respects.svg)](https://repology.org/project/pay-respects/versions)

Or if you have cargo installed:
```shell
# install from crates.io
# enabling runtime-rules is optional
cargo install pay-respects --features=runtime-rules

# clone from git and install, suitable for adding custom compile-time rules
git clone --depth 1 https://github.com/iffse/pay-respects
cd pay-respects
cargo install --path .

# compile without installing
# binary can be found at ./target/release/pay-respects
cargo build --release
```

Alternatively, you can download Linux binary from [releases](https://github.com/iffse/pay-respects/releases).

## Rule Files

See [writing rules](./rules.md) for how to write rules.

## Contributing

Current option to write rules should cover most of the cases.

We need more rule files, contributions are welcomed!

This project is hosted at various sites, choose the one that suits you best:

- [Codeberg](https://codeberg.org/iff/pay-respects)
- [GitHub](https://github.com/iffse/pay-respects)
- [GitLab](https://gitlab.com/iffse/pay-respects)

