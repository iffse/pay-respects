% PAY_RESPECTS(1) pay-respects 0.8.8 | User Commands
% iff
% April 2026

# NAME

pay-respects - Blazing fast terminal command suggestions

# SYNOPSIS

**pay-respects** *shell* [*options*]

# DESCRIPTION

pay-respects is a terminal suggestion tool that fixes your previous or current
typed command, with a sub-millisecond (<1ms) performance.

# OPTIONS

-h, --help
: Show help message and exit.

--alias
: Set a custom alias instead of `f`

--nocnf
: Disable command-not-found handler

# INITIALIZATION

## Bash / Zsh / Fish

Add the following line to your configuration file:
```sh
eval "$(pay-respects bash --alias)"
eval "$(pay-respects zsh --alias)"
pay-respects fish --alias | source
```

## Nushell / PowerShell

Add the following output to your configuration file. Replace the shell with the
actual executable name if needed.
```sh
pay-respects nu
pay-respects pwsh
```

# USAGE

Fix your previous command
```
f
```

Fix your current typed command
```
<C-x> <C-x>
```

# Examples

```
gitcommit + <Cr> + f + <Cr> --> git commit
gitcommit + <C-X> + <C-X>   --> git commit
```

# ENVIRONMENT VARIABLES

## Required

_PR_SHELL
: The binary name of the current working shell

_PR_LAST_COMMAND
: The last command

## Optional

_PR_ALIAS
: A list of aliases to commands. Separated by newlines with zsh-like
formatting, e.g. `gc=git commit`

_PR_ERROR_MSG
: Error message from the previous command. `pay-respects` will rerun previous
command to get the error message if absent

_PR_EXECUTABLES
: A space separated list of commands/executables. `pay-respects` will search for `$PATH` if absent

_PR_LIB
: Directory of modules, analogous to `PATH`. If not provided,
search in `PATH` or compile-time provided value


_PR_PACKAGE_MANAGER
: Use defined package manager instead of auto-detecting alphabetically. Empty
value disables package search functionality

_PR_MODE
: Execution mode.

## Configuration

_PR_PREFIX
: Shell prompt prefix, required for log capture from terminal multiplexers

_PR_NO_DESPERATE
: Disable desperate functions, which are slow but can give
better results

_PR_NO_CONFIG
: Don't load configurations

_PR_NO_ZOXIDE:
: Don't use zoxide

_PR_NO_MULTIPLEXER
: Equivalent of turning all the followings: `_PR_NO_TMUX`, `_PR_NO_SCREEN`, `_PR_NO_ZELLIJ`, `_PR_NO_WEZTERM`, `_PR_NO_KITTY`


# SEE ALSO

**pay-respects-rules**(5), **pay-respects-modules**(5)
