# Modules

`pay-respects` followed a very stupid approach &mdash;or better said, *Keep It Simple, Stupid*&mdash; when it comes to implementing the module / plugin system:

- Modules interacts with core program by passing **messages through processes**. In other words, we are sending necessary information to the module, so it can return the required suggestion.
- This approach is the most extendable way, as it has the least amount of limitations compared to:
	- Dynamic libraries (Safe): Requires the module to be compiled in the same compiler version as the core, which also limits the language available
	- FFI (Unsafe): Requires overloading of dynamic libraries, limits to C types, and not extendable as it's overloading a library instead of appending
	- Embedding a runtime: Unnecessary binary sizes if never used
- `pay-respects` takes the message passing approach as its core is blazing fast without observable delay so having a small overhead is acceptable. This allows:
	- **Modules in any language**: Regardless of compiled binary or interpreted scripts, just make them executable and that's a module!
	- **Extendable**: As many modules as you want
	- **Performance or ease? Both!**: Write in a compiled language if it's something computational heavy, or just use a shell script as module right away

## Creating a Module

There are 2 types of modules:

- **Standard module**: Will always run to retrieve suggestions
	- Naming convention: `_pay-respects-module-<priority>-<your module name>`
- **Fallback module**: Will only be run if no previous suggestion were found
	- **CAUTION**: Will immediately return if a suggestion is obtained
	- Naming convention: `_pay-respects-fallback-<priority>-<your module name>`

Priority is used to retrieve suggestions in a specific order by an [unstable sort](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable). Default modules have a priority of `100`.

When running your module, you will get the following environment variables:

- `_PR_SHELL`: User shell
- `_PR_COMMAND`: The command, without arguments
- `_PR_LAST_COMMAND`: Full command with arguments
- `_PR_ERROR_MSG`: Error message from the command
- `_PR_EXECUTABLES`: A space (` `) separated list of executables in `PATH`

Your module should print:

- **To `stdout`**: Only suggestions.
	- At the end of each suggestion, append `<_PR_BR>` so pay-respects knows you are either done or adding another suggestion
- **To `stderr`**: Any relevant information that should display to the user (e.g, warning for AI generated content)

An example of a shell based module that always adds a `sudo` before the command:
```sh
#!/bin/sh
echo "sudo $_PR_LAST_COMMAND"
echo "<_PR_BR>"
```

## Adding a Module

Expose your module as executable (`chmod u+x`) in `PATH`, and done!

## `LIB` directories

If exposing modules in `PATH` annoys you, you can set the `_PR_LIB` environment variable to specify directories to find the modules, separated by `:` (analogous to `PATH`):

Example in a [FHS 3.0 compliant system](https://refspecs.linuxfoundation.org/FHS_3.0/fhs/ch04s06.html):
```shell
# compile-time
export _DEF_PR_LIB="/usr/lib"
# runtime
export _PR_LIB="/usr/lib:$HOME/.local/share"
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
