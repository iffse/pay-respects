# Configuration File

Configuration file for `pay-respects` is located at:

- `$HOME/.config/pay-respects/config.toml` (*nix)
- `%APPDATA%/pay-respects/config.toml` (Windows)

## Options

All available options are listed in the following example file:
```toml
# maximum time in milliseconds for getting previous output
timeout = 3000
# your preferred command for privileges
sudo = "run0"

[package_manager]
# preferred package manager
package_manager = "pacman"

# preferred installation method, can be limited with the package manager
# available options are: System, User, Temp
install_method = "System"
```
