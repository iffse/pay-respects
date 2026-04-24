# Modules

## Creating a Module

There are 2 types of modules:

- **Standard module**: Will always run to retrieve suggestions
	- Naming convention: `_pay-respects-module-<priority>-<your module name>`
- **Fallback module**: Will only be run if no previous suggestion were found
	- **CAUTION**: Will immediately return if a suggestion is obtained
	- Naming convention: `_pay-respects-fallback-<priority>-<your module name>`

Priority is used to retrieve suggestions in a specific order after sorting. Default modules have a priority of `100`.

When running your module, you will get the following environment variables:

- `_PR_SHELL`: User shell
- `_PR_COMMAND`: The command, without arguments
- `_PR_LAST_COMMAND`: Full command with arguments
- `_PR_ERROR_MSG`: Error message from the command
- `_PR_EXECUTABLES`: A space (` `) separated list of executables in `PATH`. Limited to 100k characters, empty if exceeded.

Your module should print:

- **To `stdout`**: Only suggestions.
	- At the end of each suggestion, append `<_PR_BR>` so pay-respects knows you are either done or adding another suggestion
- **To `stderr`**: Any relevant information that should display to the user (e.g, warning for AI generated content)

An example of a shell based module that always suggest adding a `sudo` or `doas`:
```sh
#!/bin/sh
echo "sudo $_PR_LAST_COMMAND"
echo "<_PR_BR>"
echo "doas $_PR_LAST_COMMAND"
```

## Adding a Module

Expose your module as executable (`chmod u+x`) in `PATH`, and done!

## `LIB` directories

If exposing modules in `PATH` annoys you, you can set the `_PR_LIB` environment variable to specify directories to find the modules, separated by `:` or `;` (analogous to `PATH`):

Example in a [FHS 3.0 compliant system](https://refspecs.linuxfoundation.org/FHS_3.0/fhs/ch04s06.html):
```shell
# compile-time
export _DEF_PR_LIB="/usr/lib"
# runtime
export _PR_LIB="/usr/lib:$HOME/.local/share"
```
Example in Windows/DOS compliant systems
```pwsh
$env:_PR_LIB = @(
  (Join-Path $env:APPDATA "pay-respects" "modules"),
  (Join-Path $env:USERPROFILE ".cargo" "bin")
) -join ';'
```

This is not the default as there is no general way to know its value and depends on distribution (`/usr/lib`, `/usr/libexec`, or NixOS which isn't FHS compliant at all). System programs usually have a hard-coded path looking for `lib`. If you are a package maintainer for a distribution, setting this value when compiling, so it fits into your distribution standard.

If you installed the module with `cargo install`, the binary will be placed in `bin` subdirectory under Cargo's home which should be in the `PATH` anyway. Cargo has no option to place in subdirectories with other names.

The following snippet is what I have included into Arch Linux's package with workflows binaries, adding a `_PR_LIB` declaration along with initialization. The script is `/usr/bin/pay-respects` and the actual executable is located somewhere else.
```sh
#!/bin/sh
if [ "$#" -gt 1 ] && [ -z "$_PR_LIB" ]; then
	SHELL=$(basename $SHELL)
	LIB="/usr/lib/pay-respects"
	if [ "$SHELL" = "nu" ]; then
		echo "env:_PR_LIB=$LIB"
	elif [[ "$SHELL" = "pwsh" ]]; then
		echo "\$env:_PR_LIB=\"$LIB\""
	else
		echo "export _PR_LIB=$LIB"
	fi
fi
/opt/pay-respects/bin/pay-respects "$@"
```
