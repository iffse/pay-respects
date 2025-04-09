# Writing Rules

Rule files placed under [rules](./rules) in the project directory are parsed at compilation, everything is parsed to Rust code before compiling. You don't have to know the project structure nor Rust to write blazing fast rules!

For compile-time rules, if only rules are changed, cargo won't recompile the project because Rust code were intact. You will have to notify it manually by:
```shell
touch core/src/rules.rs && cargo build
```

Runtime-rules support is provided by `runtime-rules` module. Directories are searched with the following priority:

- `XDG_CONFIG_HOME`, defaults to `$HOME/.config`.
- `XDG_CONFIG_DIRS`, defaults to `/etc/xdg`.
- `XDG_DATA_DIRS`, defaults to `/usr/local/share:/usr/share`.

The actual rule file should be placed under `pay-respects/rules/`, for example: `~/.config/pay-respects/rules/cargo.toml`. Note that for runtime rules, the name of the file **MUST** match the command name.

## Syntax

Syntax of a rule file:
```toml
# the name of the command
command = "helloworld"

# you can add as many `[[match_err]]` section as you want
[[match_err]]
# Note:
# - patterns must be in lowercase without extra space characters
# - patterns should be the output with `LC_ALL=C` environment variable
# - this is a first-pass match. It should be quick so regex is not supported
pattern = [
	"pattern 1",
	"pattern 2"
]
# if pattern is matched, suggest changing the first argument to fix:
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

- `{{command}}`: All the command without any modification
- `{{command[1]}}`: The first argument of the command (the command itself has index of 0)
	- Negative values will count from reverse.
- `{{command[2:5]}}`: The second to fifth arguments
	- If any of the side is not specified, then it defaults to the start (if it is left) or the end (if it is right)
- `{{typo[2](fix1, fix2)}}`: Try to change the second argument to candidates in the parenthesis.
	- The argument in parentheses must have at least 2 values
	- Single arguments are reserved for specific matches, for instance, `path` to search all commands found in the `$PATH` environment, or the `{{shell}}` placeholder, among others
- `{{select[3](selection1, selection2)}}`: A derivative of `typo` placeholder. Will create a suggestion for each selection in the parenthesis
	- The argument in parentheses also must have at least 2 values
	- Single arguments are reserved for specific selections, for instance, `path` to search all commands found in the `$PATH` environment with the minimum shared linguistic distance, or the `{{shell}}` placeholder
	- To avoid permutations and combinations, only one instance is evaluated. If you need to apply the same selection in multiple places, use `{{selection}}`
	- Index is optional as it only has effect when using with `path`, and defaults to `0`
- `{{opt::<Regular Expression>}}`: Optional patterns captured in the command with RegEx ([see regex crate for syntax](https://docs.rs/regex-lite/latest/regex_lite/#syntax))
	- All patterns matching this placeholder will be removed from indexing
- `{{cmd::<Regular Expression>}}`: Get the matching captures from the last command
	- Unlike `{{opt}}`, this won't remove the string after matching
- `{{err::<Regular Expression}}`: Get the matching captures from the error message
- `{{shell(<shell commands>)}}`: Replace with the output of the shell command
	- Can be used along `{{typo}}` or `{{select}}` as its only argument, where each newline will be evaluated to a candidate/selection

Suggestions can have additional conditions to check. To specify conditions, add a `#[...]` at the first line (just like derive macros in Rust). Available conditions:

- `executable`: Check if the argument can be found in path
- `cmd_contains`: Check if the last user input contains the argument. Regex supported (you can't use `,` currently because it's used as condition separator)
- `err_contains`: Same as `cmd_contains` but for error message
- `length`: Check if the given command has the length of the argument
- `min_length`: Check if the given command has at least the length of the argument
- `max_length`: Check if the given command has at most the length of the argument
- `shell`: Check if the current running shell is the argument

