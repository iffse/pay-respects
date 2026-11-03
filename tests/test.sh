#!/usr/bin/env bash

set -u

ROOT=$(cd "$(dirname "$0")/.." && pwd)
CORE_BIN="$ROOT/target/debug/pay-respects"
RUNTIME_BIN="$ROOT/target/debug/_pay-respects-module-100-runtime-rules"

if [[ ! -x "$CORE_BIN" || ! -x "$RUNTIME_BIN" ]]; then
	echo "Build the workspace first: cargo build --workspace"
	exit 1
fi

CONFIG_HOME=$(mktemp -d)
CONFIG_DIRS=$(mktemp -d)
DATA_DIRS=$(mktemp -d)
trap 'rm -rf "$CONFIG_HOME" "$CONFIG_DIRS" "$DATA_DIRS"' EXIT

mkdir -p "$CONFIG_HOME/pay-respects/rules" "$CONFIG_HOME/pay-respects/scripts"
mkdir -p "$CONFIG_DIRS/pay-respects"
ln -s "$ROOT/rules" "$CONFIG_DIRS/pay-respects/rules"

for file in "$ROOT"/tests/runtime/*.toml; do
	ln -s "$file" "$CONFIG_HOME/pay-respects/rules/"
done
for file in "$ROOT"/tests/runtime/scripts/*; do
	ln -s "$file" "$CONFIG_HOME/pay-respects/scripts/"
done

export XDG_CONFIG_HOME="$CONFIG_HOME"
export XDG_CONFIG_DIRS="$CONFIG_DIRS"
export XDG_DATA_DIRS="$DATA_DIRS"
export _PR_SHELL=bash
export _PR_MODE=echo
export _PR_NO_CONFIG=1
export _PR_NO_ZOXIDE=1
export _PR_NO_MULTIPLEXER=1
export _PR_LIB=""
export _PR_EXECUTABLES=""

GREEN=$'\033[0;32m'
RED=$'\033[0;31m'
YELLOW=$'\033[0;33m'
RESET=$'\033[0m'

normalize() {
	local text=${1//<_PR_BR>/$'\n'}
	text=${text//<PR_BR>/$'\n'}
	printf '%s\n' "$text" | sed -e 's/[[:space:]]*$//' -e '/^$/d'
}

print_failure() {
	local name=$1
	local expected=$2
	local actual=$3
	echo -e "${RED}[Failed]${RESET} ${name}"
	echo "  expected:"
	printf '%s\n' "$expected" | sed 's/^/    /'
	echo "  actual:"
	printf '%s\n' "$actual" | sed 's/^/    /'
}

run_case() {
	local suite=$1
	local file=$2
	local workdir
	workdir=$(mktemp -d)

	(
		cd "$workdir" || exit 1
		source "$file" || exit 1

		local targets=${targets:-"core runtime"}
		case " $targets " in
		*" $suite "*) ;;
		*) exit 2 ;;
		esac

		export _PR_LAST_COMMAND="$command"
		export _PR_ERROR_MSG="$error"

		local stderr_file
		stderr_file=$(mktemp)
		local result
		if [[ "$suite" == "runtime" ]]; then
			export _PR_COMMAND="${command%% *}"
			result=$("$RUNTIME_BIN" 2>"$stderr_file")
		else
			result=$("$CORE_BIN" 2>"$stderr_file")
		fi
		local stderr
		stderr=$(cat "$stderr_file")
		rm -f "$stderr_file"

		if [[ -n "${expect_stderr:-}" && "$stderr" != *"$expect_stderr"* ]]; then
			print_failure "$suite: $case (stderr)" "$expect_stderr" "$stderr"
			exit 1
		fi

		if [[ "${match:-contains}" == "exact" ]]; then
			local expected
			local actual
			expected=$(normalize "$expect")
			actual=$(normalize "$result")
			if [[ "$actual" != "$expected" ]]; then
				print_failure "$suite: $case" "$expected" "$actual"
				exit 1
			fi
		else
			if [[ "$result" != *"$expect"* ]]; then
				print_failure "$suite: $case" "$expect" "$result"
				exit 1
			fi
		fi

		exit 0
	)

	local status=$?
	rm -rf "$workdir"
	return $status
}

passed=0
failed=0
skipped=0

echo "Starting suggestion tests..."
echo "-----------------------------------------"

for suite in core runtime; do
	for file in "$ROOT"/tests/cases/*; do
		run_case "$suite" "$file"
		status=$?
		case $status in
		0) passed=$((passed + 1)) ;;
		2) skipped=$((skipped + 1)) ;;
		*) failed=$((failed + 1)) ;;
		esac
	done
done

echo "-----------------------------------------"
echo -e "${GREEN}Passed${RESET}: ${passed}  ${RED}Failed${RESET}: ${failed}  ${YELLOW}Skipped${RESET}: ${skipped}"

if [[ $failed -ne 0 ]]; then
	exit 1
fi
