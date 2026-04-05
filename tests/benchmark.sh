#!/usr/bin/bash

export PR=$(realpath ../target/release/pay-respects)
export _PR_SHELL="bash"
export _PR_LIB=""
export _PR_MODE="echo"
export TMP=$(mktemp -d)

export _PR_NO_ZOXIDE=1
export _PR_NO_DESPERATE=1
export _PR_NO_CONFIG=1
export _PR_NO_MULTIPLEXER=1

benchmark() {
	hyperfine --warmup 10 \
		--shell none \
		"$1"
}

run_test() {
	export _PR_LAST_COMMAND=$command
	export _PR_ERROR_MSG=$error
	echo "Benchmark: $case"
	benchmark $PR
}

run_case() {
	(
		source $1
		run_test
	)

}

main() {
	echo "Starting benchmarking tests..."
	echo "-----------------------------------------"
	WORKDIR=$(pwd)
	cd $TMP
	echo "Benchmark: Initialization"
	echo "$PR bash"
	benchmark "$PR bash"
	for case in $WORKDIR/cases/*; do
		run_case $case
		rm -rf ./*
	done

	rm -rf $TMP
}

main "$@"
