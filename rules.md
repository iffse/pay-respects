# Writing Rules

Rule files placed under [rules](./rules) in the project directory are parsed at compilation, everything is parsed to Rust code before compiling. You don't have to know the project structure nor Rust to write blazing fast rules!

For compile-time rules, if only rules are changed, cargo won't recompile the project because Rust code were intact. You will have to notify it manually by:
```shell
touch src/rules.rs && cargo build
```

Runtime rules directories are searched with the following priority:

- `XDG_CONFIG_HOME`, defaults to `$HOME/.config`.
- `XDG_CONFIG_DIRS`, defaults to `/etc/xdg`.
- `XDG_DATA_DIRS`, defaults to `/usr/local/share:/usr/share`.

The actual rule file should be placed under `pay-respects/rules/`, for example: `~/.config/pay-respects/rules/cargo.toml`. Note that for runtime rules, the name of the file **MUST** match the command name.

Syntax of a rule file:
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
# - the `sudo` is found as executable
# - the last command does not contain `sudo`
suggest = [
'''
#[executable(sudo), !cmd_contains(sudo)]
sudo {{command}} '''
]
```

The placeholder is evaluated as following:

- `{{command}}`: All the command without any modification.
- `{{command[1]}}`: The first argument of the command (the command itself has index of 0). Negative values will count from reverse.
- `{{command[2:5]}}`: The second to fifth arguments. If any of the side is not specified, then it defaults to the start (if it is left) or the end (if it is right).
- `{{typo[2](fix1, fix2)}}`: This will try to change the second argument to candidates in the parenthesis. The argument in parentheses must have at least 2 values. Single arguments are reserved for specific matches, for instance, `path` to search all commands found in the `$PATH` environment, or the `{{shell}}` placeholder, among others.
- `{{opt::<Regular Expression>}}`: Optional patterns captured in the command with RegEx ([see regex crate for syntax](https://docs.rs/regex-lite/latest/regex_lite/#syntax)). Note that all patterns matching this placeholder will be removed from indexing.
- `{{cmd::<Regular Expression>}}`: Get the matching captures from the last command. Unlike `{{opt}}`, this won't remove the string after matching
- `{{err::<Regular Expression}}`: Get the matching captures from the error message.
- `{{shell(<shell commands>)}}`: Replace with the output of the shell command. This placeholder can be used along `{{typo}}` as its only argument, where each newline will be evaluated to a candidate.

Suggestions can have additional conditions to check. To specify conditions, add a `#[...]` at the first line (just like derive macros in Rust). Available conditions:

- `executable`: Check whether the argument can be found as executable (`command -v arg`).
- `cmd_contains`: Check whether the last user input contains the argument.
- `err_contains`: Check whether the error of the command contains the argument.
- `length`: Check whether the given command has the length of the argument.
- `min_length`: Check whether the given command has at least the length of the argument.
- `max_length`: Check whether the given command has at most the length of the argument.
- `shell`: Check if the current running shell is the argument.

For locale other than English, be aware that patterns should be the output having `LC_ALL=C` environment variable.

