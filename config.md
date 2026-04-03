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
# Maximum time in milliseconds for getting previous output
timeout = 3000

# Apply existing rules to a set of commands. Every set will use the rule
# corresponding to the first entry
merge_commands = [
	[ls, exa], # both using rules corresponding to `ls`
	[grep, rg], # both using rules corresponding to `grep`
]

# How suggestions are evaluated after being confirmed
# Options can be:
# - Internal: Commands are evaluated inside `pay-respects`
# - Shell: Current working shell is responsible for execution
eval_method = "Internal"

[package_manager]
# Preferred package manager
package_manager = "pacman"

# Preferred installation method, can be limited by the package manager
# Available options are:
# - System
# - Shell (nix and guix only)
install_method = "System"

# Algorithm used for fuzzy searching
# Options:
# - TrigramDamerauLevenshtein: More computationally expensive
# - DamerauLevenshtein: Fast
search_type = "TrigramDamerauLevenshtein"

# Minimum characters required to start seaching
# Avoids false positive when input strings are too short
seach_threshold = 3

# Minimum score for the result to be valid: [0,1]
[trigram]
minimum_score = 0.271828182845904523536028747135266250

# Configuring the linguistic distance (Damerau-Levenshtein)
# - Percentage: How many characters can be different in the suggestion, rounded
#   to the nearest integer.
# - Threshold: How many characters are required for the reference string to
#   start searching to avoid false positives when the reference string is too
#   short.
# - Max: Maximum distance allowed between the reference and the suggestion.
# - Min: Minimum strating distance required. Seach only starts if the distance
#   obtained after the percentage calculation is above or equal to this value.
[dl_distance]
percentage = 27.1828182845904523536028747135266250
max = 5
min = 1
```
