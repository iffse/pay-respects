# Pay Respect

Typed a wrong command? Pay Respect will try to correct your wrong console command simply by pressing `F`!

## How to Pay Respect

The binary is named `pay-respect`, by adding an alias to your shell
configuration:
``` shell
alias f="pay_respect"
```
You can now **press `F` to Pay Respect**!

## Rule Files

Rule files are parsed at compilation. What actually gets compiled is a HashMap that contains patterns and suggestions for a specific command.

Syntax of a rule file (placed under [rules](./rules)):
```toml
# this field should be the name of the command
command = "world"

# you can add as much section of this as you like
[[match_output]]
# the suggestion of this section will be used for the following patterns of the command output
pattern = [
	"pattern 1",
	"pattern 2",
]
# this will change the first argument to `fix`, while keeping the rest intact
suggest = "{{command[0]}} fix {{command[2:]}}"

[[match_output]]
pattern = [
	"pattern 1",
]
# this will add a `sudo` before the command, without touching the rest
suggest = "sudo {{command}}"
```

The placeholder is evaluated as following:

- `{{command}}`: All the command without any modification
- `{{command[1]}}`: The first argument of the command (the command itself has index of 0)
- `{{command[2:5]}}`: The second to fifth arguments. If any of the side is not specified, them it defaults to the start (if it is left) or the end (if it is right).

## Current Progress

We need more rule files!

