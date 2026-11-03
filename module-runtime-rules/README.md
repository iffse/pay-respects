# Runtime Rules Module

Adds runtime rules to [pay-respects].

The syntax matches [upstream's compile-time rules].

[pay-respects]: https://codeberg.org/iff/pay-respects
[upstream's compile-time rules]: https://codeberg.org/iff/pay-respects/src/branch/main/rules.md

Rules are loaded from these directories:

- `XDG_CONFIG_HOME`, default `$HOME/.config`
- `XDG_CONFIG_DIRS`, default `/etc/xdg`
- `XDG_DATA_DIRS`, default `/usr/local/share:/usr/share`

Place rule files under `pay-respects/rules/`. For example,
`~/.config/pay-respects/rules/cargo.toml`. The file name must match the command
name. This avoids parsing rules that will never match.

`_PR_GENERAL.toml` is always parsed.
