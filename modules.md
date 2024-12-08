# Modules

`pay-respects` followed a very stupid approach ---or better said, *Keep It Simple, Stupid*--- when it comes to implementing the module / plugin system:

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

Priority is used to retrieve suggestions in a specific order by an [unstable sort](https://doc.rust-lang.org/std/primitive.slice.html#method.sort_unstable) (although they will always be after compile-time matches). Default modules have a priority of `100`.

When running your module, you will get the following environment variables:

- `_PR_SHELL`: User shell
- `_PR_COMMAND`: The command, without arguments
- `_PR_LAST_COMMAND`: Full command with arguments
- `_PR_ERROR_MSG`: Error message from the command
- `_PR_EXECUTABLES`: A comma (`,`) separated list of executables in `PATH`

Your module should return:

- **To `stdout`**: Only suggestions.
	- At the end of each suggestion, append `<_PR_BR>` so pay-respects knows you are either done or adding another suggestion
- **To `stderr`**: Any relevant information that should display to the user (e.g, warning for AI generated content)

## Adding a Module

Expose your module as executable (`chmod u+x`) in `PATH`, and done!

