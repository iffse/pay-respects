% PAY_RESPECTS(5) pay-respects 0.8.3 | Configuration File
% iff
% April 2026

# Writing Rules

Rule files placed under [rules](./rules) in the project directory are parsed at
compilation, everything is parsed to Rust code before compiling. You don't have
to know the project structure nor Rust to write blazing fast rules!

Runtime-rules support is provided by `runtime-rules` module. Directories are
searched with the following priority:

- `XDG_CONFIG_HOME`, defaults to `$HOME/.config`.
- `XDG_CONFIG_DIRS`, defaults to `/etc/xdg`.
- `XDG_DATA_DIRS`, defaults to `/usr/local/share:/usr/share`.

The actual rule file should be placed under `pay-respects/rules/`, for example:
`~/.config/pay-respects/rules/cargo.toml`. Note that for runtime rules, the
name of the file **MUST** match the command name. Except `_PR_GENERAL.toml`,
that is always parsed.

## Syntax

Syntax of a rule file:
```toml
# The name of the command
# If multiple commands could share the same rules, add it to extends
command = "hello"
extends = ["goodbye"]

# You can add as many `[[match_err]]` section as you want
[[match_err]]
# Note:
# - Patterns should be the output with `LC_ALL=C` environment variable
# - This is a first-pass match. It should be quick so regex is not supported
# - This field is optional, always match if omitted
pattern = [
	"pattern 1",
	"pattern 2"
]
# If pattern is matched, suggest changing the first argument to fix:
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

## Rust Functions

You can also write in Rust if the rule is very complex. This is only allowed
during compilation, by previously defining your function in
[`rules_functions.rs`](./core/src/rules_function.rs):
```toml
[[match_err]]
pattern = [
	"pattern 1"
]
suggest = [
'''
#[FUNCTION]
MyRustFunction '''
]
```

## Placeholders

The placeholder is evaluated as following:

- `{{command}}`: All the command without any modification
- `{{command[1]}}`: The first argument of the command (the command itself has
index of 0)
	- Negative values will count from reverse.
- `{{command[2:5]}}`: The second to fifth arguments
	- If any of the side is not specified, then it defaults to the start (if it
	is left) or the end (if it is right)
- `{{typo[2](fix1, fix2)}}`: Try to change the second argument to candidates in
the parenthesis.
	- The argument in parentheses must have at least 2 values
	- Single arguments are reserved for specific matches, for instance, `path` to
	search all commands found in the `$PATH` environment, or the `{{shell}}`
	placeholder, among others
- `{{select[3](selection1, selection2)}}`: A derivative of `typo` placeholder.
Will create a suggestion for each selection in the parenthesis
	- The argument in parentheses also must have at least 2 values
	- Single arguments are reserved for specific selections, for instance, `path`
	to search all commands found in the `$PATH` environment with the minimum
	shared linguistic distance, or the `{{shell}}` placeholder
	- To avoid permutations and combinations, only one instance is evaluated. If
	you need to apply the same selection in multiple places, use
	`{{selection}}`
	- Index is optional as it only has effect when using with `path`, and
	defaults to `0`
- `{{opt::<Regular Expression>}}`: Optional patterns captured in the command
with RegEx ([see regex crate for syntax])
	- All patterns matching this placeholder will be removed from indexing
- `{{cmd::<Regular Expression>}}`: Get the matching captures from the last command
	- Unlike `{{opt}}`, this won't remove the string after matching
- `{{err::<Regular Expression}}`: Get the matching captures from the error
	message (**case-sensitive**)
- `{{shell(<shell commands>)}}`: Replace with the output of the shell command
	- Can be used along `{{typo}}` or `{{select}}` as its only argument, where
	each newline will be evaluated to a candidate/selection

[see regex crate for syntax]: https://docs.rs/regex-lite/latest/regex_lite/#syntax

## Conditions

Suggestions can have additional conditions to check. To specify conditions, add
a `#[...]` at the first line (just like derive macros in Rust). Available
conditions:

- `executable`: Check if the argument can be found in path
- `cmd_contains`: Check if the last user input contains the argument. Regex
supported (using `,` requires escaping, i.e. `\,`)
- `err_contains`: Same as `cmd_contains` but for error message (all lowercase)
- `length`: Check if the given command has the length of the argument
- `min_length`: Check if the given command has at least the length of the argument
- `max_length`: Check if the given command has at most the length of the argument
- `shell`: Check if the current running shell is the argument

## Identifiers

Identifiers are used to reuse rules for specific purposes:

- `INLINE`: Reuse the rule for `inline` mode. Patterns are ignored.

## Other Considerations

When suggesting a chained command with `&&`, try to break it into multiple lines.

- More readable if the commands are long
- Automatic conversion to compatible syntax where `&&` is unavailable (e.g. nushell)

Example:
```toml
suggest = [
'''
command1 &&
command2 '''
]
```

# SEE ALSO

**pay-respects**(1), **pay-respects-modules**(5)
