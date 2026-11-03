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
# Preferred command for privileged acesses
privilege = "sudo"

# Maximum time in milliseconds for getting previous output
timeout = 3000

# Apply existing rules to a set of commands. Every set will use the rule
# corresponding to the first entry
merge_commands = [
	["ls", "exa"], # both using rules corresponding to `ls`
	["grep", "rg"], # both using rules corresponding to `grep`
]

# Commands that won't return any message when run,
# which is most of the interactive commands
blocking_commands = ["vim", "nano"]

# How suggestions are evaluated after being confirmed
# Options can be:
# - Internal: Commands are evaluated inside `pay-respects`
# - Shell: Current working shell is responsible for execution
eval_method = "Internal"

# Algorithm used for fuzzy searching
# Options:
# - TrigramDamerauLevenshtein: More computationally expensive
# - DamerauLevenshtein: Fast
search_type = "TrigramDamerauLevenshtein"

# Minimum characters required to start seaching
# Avoids false positive when input strings are too short
search_threshold = 3

# Selection interface
[ui]
# Print the message above the list
prelude = true
# Print the line with the key hints
hint = true

# Interface colors. Accepts color names ("red", "bright blue") and hex codes
# ("#ff8800"). The selection interface, the highlighted suggestions and the
# messages all use these.
[colors]
# Parts to correct and error messages
wrong = "red"
# Confirmations and successful states
correct = "green"
# Unchanged parts and interface elements
accent = "blue"
# Normal text. Set to "none" to keep the terminal's default foreground
normal = "none"

# Minimum score for the result to be valid: [0,1]
[trigram]
minimum_score = 0.5

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

[package_manager]
# Preferred package manager
package_manager = "pacman"

# Preferred installation method, can be limited by the package manager
# Available options are:
# - System
# - Shell (nix and guix only)
install_method = "System"

# Configuration for the `request-ai` module
[module.request-ai]
# Enable AI suggestions.
# If unset, and no API key is configured, pay-respects asks whether to use the
# built-in API key. Set to false to disable the module.
enabled = true

# API key. Leave unset to use the built-in API key.
api_key = "..."

# Any OpenAI-compatible API endpoint
api_url = "https://api.openai.com/v1/chat/completions"

# Model name
model = "gpt-4o-mini"

# Extra JSON fields to include in the request
extra = '"temperature" = 0.5'

# Extra JSON object for the `extra_body` field
extra_body = '{"chat_template_kwargs": {"enable_thinking": false}}'

# Locale in which the AI explains the suggestion
locale = "en-US"

# Additional prompt
additional_prompt = "User's environment is Zsh running in Arch Linux."
```

Environment variables for the AI module take precedence over the
configuration file and are documented in
[module-request-ai](./module-request-ai/README.md).
