# Version 1 Rules (Deprecated)

Version 1 still works at runtime. New rules should use
[version 2](./rules.md#version-2).

## Syntax

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

Fields:

- `command`: Main command.
- `extends`: Aliases that share the rule.
- `[[match_err]]`: One error match block.
- `pattern`: Optional error substrings.
- `suggest`: Suggestion strings.
  - Conditions go in a `#[...]` block at the start.
  - Commas inside a regex must be escaped as `\,`.
  - `INLINE` allows the suggestion in inline mode.
  - `FUNCTION` calls a Rust function from
    [`rules_function.rs`](./core/src/rules_function.rs).

Placeholders and conditions use the same syntax as version 2. See
[the current rules documentation](./rules.md#placeholders).

## Migration

Use the built-in converter:

```sh
pay-respects --migrate-rules ~/.config/pay-respects/rules/git.toml > git.toml
```

The command writes the converted file to `stdout`. It accepts one file at a
time. Comments are not preserved. Review the output before replacing the old
file.
