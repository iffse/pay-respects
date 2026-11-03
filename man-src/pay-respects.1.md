% PAY_RESPECTS(1) pay-respects 0.9.0 | User Commands
% iff
% April 2026

# NAME

pay-respects - Blazing fast terminal command suggestions

# SYNOPSIS

**pay-respects** [*shell-binary*] [*options*]
**pay-respects** --migrate-rules *file*

# DESCRIPTION

pay-respects is a terminal suggestion tool that fixes your previous or current
typed command, with a sub-millisecond (<1ms) performance.

The optional positional argument is the shell executable. Its type is detected
from the name. Use `--shell-type` when the name does not contain a known shell
name, for example `pay-respects zsh-fork --shell-type zsh`.

# OPTIONS

-h, --help
: Show help message and exit.

--alias
: Set a custom alias instead of `f`

-s, --shell-type &lt;type&gt;
: Shell type when the binary name differs, such as `zsh-fork`. The type is
detected from the binary name when omitted.

--migrate-rules &lt;file&gt;
: Convert a version 1 rule file to version 2 on `stdout`.

--nocnf
: Disable command-not-found handler

# INITIALIZATION

## Bash / Zsh / Fish / Dash

Add the following line to your configuration file:
```sh
eval "$(pay-respects bash --alias)"
eval "$(pay-respects zsh --alias)"
pay-respects fish --alias | source
eval "$(pay-respects dash --alias)"
```

Dash has no inline mode or command-not-found handler.

## Xonsh

Add the following line to `~/.xonshrc`:
```xsh
execx($(pay-respects xonsh --alias), 'exec', __xonsh__.ctx, filename='pay-respects')
```

Inline mode requires prompt_toolkit.

## Elvish

Add the following line to `~/.config/elvish/rc.elv`:
```elv
eval (pay-respects elvish --alias | slurp)
```

Type `f` to suggest a fix. Inline mode is bound to `Ctrl-X`. Elvish has no
command-not-found handler.

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

Put the selected fix back into the prompt, to be edited with the shell's own key
bindings, on fish and Nushell
```
f + <e>
```

On other shells, or with `[ui] prompt_edit = false`, the fix is opened in
`$VISUAL`/`$EDITOR` instead and run when saved.

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

_PR_SHELL_TYPE
: The shell type when it differs from the binary name. Detected from
`_PR_SHELL` if absent

_PR_ALIAS
: A list of aliases to commands. Separated by newlines with zsh-like
formatting, e.g. `gc=git commit`

_PR_ERROR_MSG
: Error message from the previous command. `pay-respects` will rerun previous
command to get the error message if absent

_PR_EXECUTABLES
: A space separated list of commands/executables. `pay-respects` will search for `$PATH` if absent

_PR_LOG
: Debug builds only. Debug log destination. Use `stderr`, a file path, or
`1` for `pay-respects.log` in the current directory. Unset disables logs

_PR_LOG_LEVEL
: Maximum debug log level, default `5`

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
