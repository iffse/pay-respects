# Configuration File

User configuration file for `pay-respects` is located at:

- `$HOME/.config/pay-respects/config.toml` (*nix)
- `%APPDATA%/pay-respects/config.toml` (Windows)

Directories for system-wide configuration, which fields can be overwritten
individually by the user configuration file are searched on:

- `$XDG_CONFIG_DIRS` (*nix)
	- If not set, `/etc/xdg`
- `$PROGRAMDATA` (Windows)
	- If not set, `C:\ProgramData`

With the same directory structure, e.g. `/etc/xdg/pay-respects/config.toml`

## Options

All available options are listed in the following example file:
```toml
# maximum time in milliseconds for getting previous output
timeout = 3000

# how suggestions are evaluated after being confirmed
# options can be:
# - Internal: commands are evaluated inside `pay-respects`
# - Shell: current working shell is responsible for execution
eval_method = "Internal"

[package_manager]
# preferred package manager
package_manager = "pacman"

# preferred installation method, can be limited by the package manager
# available options are:
# - System
# - Shell (nix and guix only)
install_method = "System"
```
