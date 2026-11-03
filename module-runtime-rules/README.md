# Runtime Rules Module

Adds runtime rules to [pay-respects].

The syntax matches [upstream's compile-time rules]. Version 2 is preferred.
Version 1 still works, but the module prints a deprecation warning when it loads
one.

[pay-respects]: https://codeberg.org/iff/pay-respects
[upstream's compile-time rules]: https://codeberg.org/iff/pay-respects/src/branch/main/rules.md

Rules are loaded from these directories:

- `XDG_CONFIG_HOME`, default `$HOME/.config`
- `XDG_CONFIG_DIRS`, default `/etc/xdg`
- `XDG_DATA_DIRS`, default `/usr/local/share:/usr/share`

Place rule files under `pay-respects/rules/`. For example,
`~/.config/pay-respects/rules/cargo.toml`. Name the file after the main
command. The `commands` field lists aliases.

`_PR_GENERAL.toml` is always parsed.

Script suggestions run with an external interpreter. Relative script paths are
searched under `pay-respects/scripts/` next to the rule directories, then beside
the rule file. Rust function suggestions are skipped because they only exist in
the compile-time rules.
