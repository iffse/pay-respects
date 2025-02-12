# Runtime Rules Module

Module for [pay-respects](https://codeberg.org/iff/pay-respects) which allows you to parse rules at runtime.

Syntax is currently 100% compatible with [upstream's compile-time rules](https://codeberg.org/iff/pay-respects/src/branch/main/rules.md).

Rules are searched in these directories:

- `XDG_CONFIG_HOME`, defaults to `$HOME/.config`.
- `XDG_CONFIG_DIRS`, defaults to `/etc/xdg`.
- `XDG_DATA_DIRS`, defaults to `/usr/local/share:/usr/share`.

The actual rule file should be placed under `pay-respects/rules/`, for example: `~/.config/pay-respects/rules/cargo.toml`. To avoid parsing unnecessary rules, the name of the file **MUST** match the command name.
