# Suggestion Test Cases

Each file is a Bash snippet. The runner sources it, sets `_PR_LAST_COMMAND` and
`_PR_ERROR_MSG`, then runs the core binary or the runtime rules module.

The letter prefix is the category. Keep numbers increasing inside a category.

## A: Core behavior

Tests for matching, placeholders, typo correction, privilege, blocking, and
general rules. These cases do not belong to a single command rule.

## B: Command rules

Tests for one command or tool. Examples include `cd`, `git`, `brew`, `grep`,
`go`, `python`, `docker`, and `terraform`.

## C: Fallback recovery

Tests for the desperate fallback path. These cases only run in core, because
the runtime rules module does not implement that path.

## D: Runtime rules

Tests for the runtime rules module. They cover aliases, variables, scripts, and
version 1 warnings.

## Case Fields

- `case`: Test name shown on failure.
- `command`: Full command to fix.
- `error`: Error output.
- `expect`: Expected text. Set `match="exact"` for a full match.
- `targets`: Space separated `core` and `runtime`. Defaults to both.
- `expect_stderr`: Optional text that must appear on stderr.
