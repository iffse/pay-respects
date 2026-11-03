# Writing Rules

Rule files live under [rules](./rules). Core parses them at build time. The
`runtime-rules` module reads them from the user configuration at run time.

New rules should use [version 2](#version-2). Version 1 still works, but the
runtime module prints a warning when it loads one.

## Locations

Runtime rules are loaded from:

- `XDG_CONFIG_HOME`, default `$HOME/.config`
- `XDG_CONFIG_DIRS`, default `/etc/xdg`
- `XDG_DATA_DIRS`, default `/usr/local/share:/usr/share`

Place files under `pay-respects/rules/`, for example
`~/.config/pay-respects/rules/cargo.toml`. Name the file after the main
command. `_PR_GENERAL.toml` is always parsed.

## Version 2

```toml
version = 2
commands = ["hello", "hi"]

[[match]]
# Error patterns are optional. A match without patterns always runs.
error = [
	"pattern 1",
	"pattern 2",
]
vars = { dir = "{{command[1]}}" }

[[match.suggest]]
# Conditions are checked before the suggestion is used.
when = [
	"executable(sudo)",
	"!cmd_contains(sudo)",
]
run = "sudo {{command}}"

[[match.suggest]]
when = ["length(2)"]
inline = true
run = "{{command[0]}} fix {{command[1]}}"

[[match.suggest]]
run = "cd {{$dir}}"

[[match.suggest]]
function = "MyRustFunction"

[[match.suggest]]
script = { interpreter = "/usr/bin/python3", file = "git-fix.py" }
```

Fields:

- `version`: `2` for this format.
- `commands`: Command names that share the file. Aliases go here.
- `error`: Case and whitespace insensitive substrings from the error output.
	Omit it to always match.
- `vars`: Optional variables for the match. Use a table or an array of
	`[name, value]` pairs.
- `when`: Conditions, one string per condition. Conditions are ANDed.
- `inline`: Also allow the suggestion in inline mode. Patterns are ignored in
	inline mode.
- `run`: One suggestion template. Use another `[[match.suggest]]` block for
	another suggestion.
- `function`: Call a Rust function from
	[`rules_function.rs`](./core/src/rules_function.rs). Runtime rules skip it
	because they cannot run Rust functions.
- `script`: Run a script with an external interpreter. Use exactly one of
	`run`, `function`, or `script`.

Variables are expanded before the suggestion template is parsed. A value can
use placeholders or other variables, so `{{$dir}}` also works inside another
placeholder. Unknown variables and cycles fail at parse time.

The pair form is useful when variable order matters:

```toml
[[match]]
vars = [
	["cmd", "{{command[0]}}"],
	["arg", "{{command[1]}}"],
]
```

Each condition is its own string. A comma inside a regex does not need
escaping.

## Scripts

`script` runs a script with an external interpreter. Scripts are only available
in runtime rules. Core rejects them at build time.

```toml
[[match.suggest]]
script = { interpreter = "/usr/bin/python3", file = "git-fix.py" }
```

`file` can be absolute. A relative path is searched in these directories:

- `XDG_CONFIG_HOME/pay-respects/scripts`, default `$HOME/.config/pay-respects/scripts`
- `XDG_CONFIG_DIRS/pay-respects/scripts`, default `/etc/xdg/pay-respects/scripts`
- `XDG_DATA_DIRS/pay-respects/scripts`, default
	`/usr/local/share/pay-respects/scripts:/usr/share/pay-respects/scripts`
- The directory of the rule file

The interpreter is run with the script file as its only argument. This works
with Python, Lua, Node, shell, and other interpreters that accept a file path.

The script gets the same environment variables as a module:

- `_PR_COMMAND`: Command without arguments
- `_PR_LAST_COMMAND`: Full command
- `_PR_ERROR_MSG`: Error message
- `_PR_SHELL`: Current shell
- `_PR_EXECUTABLES`: Executables separated by spaces. Empty when the list is
	too large.

Print suggestions to `stdout`. Separate multiple suggestions with `<_PR_BR>`.
Without the separator, each non-empty line becomes one suggestion.

Variables are substituted in `interpreter` and `file` before they are used.

## Version 1 (deprecated)

```toml
command = "hello"
extends = ["goodbye"]

[[match_err]]
pattern = [
	"pattern 1",
]
suggest = [
'''
#[executable(sudo), !cmd_contains(sudo)]
sudo {{command}} '''
]
```

In version 1, `extends` lists aliases. Conditions go inside a `#[...]` block at
the start of the suggestion. Commas inside a regex must be escaped as `\,`.

Version 1 remains supported at run time. Migrate to version 2 before it is
removed.

## Migrating from Version 1

Use the built-in converter:

```sh
pay-respects --migrate-rules ~/.config/pay-respects/rules/git.toml > git.toml
```

The command writes the converted file to `stdout`. It accepts one file at a
time. Comments are not preserved. Review the output before replacing the old
file.

## Placeholders

- `{{command}}`: The full command without changes
- `{{command[1]}}`: The first argument. The command itself is index 0.
	- Negative values count from the end.
- `{{command[2:5]}}`: Arguments from index 2 through index 5.
	- An empty side starts at the beginning or ends at the last argument.
- `{{typo[2](fix1, fix2)}}`: Replace the second argument with the closest
	candidate.
	- A candidate list needs at least two values.
	- `path` searches executables in `$PATH`.
	- `file` searches files on disk.
	- `{{shell(<command>)}}` uses each output line as a candidate.
- `{{select[3](one, two)}}`: Create one suggestion per selection.
	- Only one `select` is allowed in a template.
	- Use `{{selection}}` to place the same selection in another position.
	- The index is optional and only affects `path`. It defaults to `0`.
- `{{opt::<Regular Expression>}}`: Remove the captured patterns from the
	command before later placeholders run.
- `{{cmd::<Regular Expression>}}`: Insert the captures from the command.
- `{{err::<Regular Expression>}}`: Insert the captures from the error message.
	Matching is case-sensitive.
- `{{shell(<command>)}}`: Insert the output of a shell command.
	- Inside `typo` or `select`, each output line becomes a candidate or
		selection.

Regex syntax follows [regex-lite].

[regex-lite]: https://docs.rs/regex-lite/latest/regex_lite/#syntax

## Conditions

- `executable`: The argument is in `$PATH`
- `cmd_contains`: The last command matches the regex
- `err_contains`: The error message matches the regex, ignoring case and
	whitespace
- `length`: The command has exactly this many arguments
- `min_length`: The command has at least this many arguments
- `max_length`: The command has at most this many arguments
- `shell`: The current shell equals the argument

Prefix a condition with `!` to negate it.

## Other Considerations

When a suggestion chains commands with `&&`, prefer multiple lines.

- Long commands read better that way.
- The shell integration can convert `&&` where the shell does not support it,
	for example Nushell.

Example:

```toml
[[match.suggest]]
run = '''
command1 &&
command2 '''
```
