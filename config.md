# Configuration File

Configuration file for `pay-respects` is located at:

- `$HOME/.config/pay-respects/config.toml` (*nix)
- `%APPDATA%/pay-respects/config.toml` (Windows)

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
