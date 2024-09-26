# Pay Respects

Typed a wrong command? Pay Respects will try to correct your wrong console command by simply pressing `F`!

- 🚀 **Blazing fast suggestion**: You won't notice any delay for asking suggestions!
- ✏️ **Easy to write rules**: You don't need to know Rust. The rules are written in a TOML file that is simple to work with and evaluated to Rust code upon compilation!
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

## Installing

If you are using Arch Linux, you can install from AUR directly:
```shell
paru -S pay-respects     # compile from source
paru -S pay-respects-bin # binary version
```

Or if you have cargo installed:
```shell
# install from crates.io
cargo install pay-respects

# clone from git and install, suitable for adding custom rules
git clone --depth 1 https://github.com/iffse/pay-respects
cd pay-respects
cargo install --path .

# compile without installing
# binary can be found at ./target/release/pay-respects
cargo build --release
```

Alternatively, you can download Linux binary from [releases](https://github.com/iffse/pay-respects/releases).

## Rule Files

Rule files are parsed at compilation. Everything in rule files is converted to Rust code before compiling. You don't have to know the project structure nor Rust to write the rules!

If only rules are changed, cargo won't recompile the project because Rust code were intact. You will have to notify it manually by:
```shell
touch src/suggestions.rs && cargo build
```

Syntax of a rule file (will be read by simply placing the file under [rules](./rules)):
```toml
# this field should be the name of the command
command = "world"

# you can add as many `[[match_err]]` section as you want
[[match_err]]
# the suggestion of this section will be used for the following patterns of the error output
# note that the error is formatted to lowercase
pattern = [
	"pattern 1",
	"pattern 2"
]
# this will change the first argument to `fix`, while keeping the rest intact
suggest = [
'''
{{command[0]}} fix {{command[2:]}} '''
]

[[match_err]]
pattern = [
	"pattern 1"
]
# this will add a `sudo` before the command if:
# - the `sudo` is found by `command -v`
# - the last command does not contain `sudo`
suggest = [
'''
#[executable(sudo), !cmd_contains(sudo)]
sudo {{command}} '''
]
```

The placeholder is evaluated as following:

- `{{command}}`: All the command without any modification
- `{{command[1]}}`: The first argument of the command (the command itself has index of 0)
- `{{command[2:5]}}`: The second to fifth arguments. If any of the side is not specified, then it defaults to the start (if it is left) or the end (if it is right).
- `{{typo[2](fix1, fix2)}}`: This will try to change the second argument to candidates in the parenthesis. The argument in parentheses must have at least 2 values. Single arguments are reserved for specific matches, for instance, `path` to search all commands found in the `$PATH` environment, or the `{{shell}}` placeholder, among others.
- `{{opt::<Regular Expression>}}`: Optional patterns found in the command with RegEx (see RegEx crate for syntax). Note that all patterns matching this placeholder will not take place when indexing.
- `{{cmd::<Regular Expression>}}`: Get the matching pattern from the last command. Unlike `{{opt}}`, this won't remove the string after matching
- `{{err::<Regular Expression}}`: Get the matching patterns from the error message.
- `{{shell(<shell commands>)}}`: Replace with the output of the shell command. This placeholder can be used along `{{typo}}` as its only argument, where each newline will be evaluated to a candidate.

Suggestions can have additional conditions to check. To specify conditions, add a `#[...]` at the first line (just like derive macros in Rust). Available conditions:

- `executable`: Check whether the argument can be found by `which`
- `cmd_contains`: Check whether the last user input contains the argument
- `err_contains`: Check whether the error of the command contains the argument
- `length`: Check whether the given command has the length of the argument
- `min_length`: Check whether the given command has at least the length of the argument
- `max_length`: Check whether the given command has at most the length of the argument

For locale other than English, be aware that patterns should be the output having `LC_ALL=C` environment variable.

## Current Progress

Current option to write rules should cover most of the cases.

We need more rule files, contributions are welcomed!

