#!/usr/bin/bash

export PR=$(realpath ../target/debug/pay-respects)
export _PR_SHELL="bash"
export _PR_LIB=""
export _PR_MODE="echo"
export TMP=$(mktemp -d)

# can take a while in debug builds
# required for desperate checks
# export _PR_NO_DESPERATE=1

# zoxide is too slow
export _PR_NO_ZOXIDE=1

export _PR_NO_CONFIG=1
export _PR_NO_MULTIPLEXER=1

PASSED=0
FAILED=0
export green='\033[0;32m'
export red='\033[0;31m'
export reset='\033[0m'

run_test() {
	export _PR_LAST_COMMAND=$command
	export _PR_ERROR_MSG=$error
	result=$($PR 2>/dev/null)
	if [[ $result == *"$expect"* ]]; then
		echo -e "${green}[Passed]${reset}: $case"
		return 0
	else
		echo -e "${red}[Failed]${reset}: $case"
		echo -e "\texpected: $expect"
		echo -e "\tgot: $result"
		return 1
	fi
}

run_case() {
	(
		source $1
		run_test
	)

	if [[ $? == 0 ]]; then
		PASSED=$((PASSED + 1))
	else
		FAILED=$((FAILED + 1))
	fi
}

main() {
	echo "Starting suggestion tests..."
	echo "-----------------------------------------"
	WORKDIR=$(pwd)
	cd $TMP
	for case in $WORKDIR/cases/*; do
		run_case $case
		rm -rf ./*
	done
	
	echo "-----------------------------------------"
	echo -en "${green}Passed${reset}: $PASSED\t"
	echo -en "${red}Failed${reset}: $FAILED\t"
	echo -e "Total: $((PASSED + FAILED))"

	rm -rf $TMP
	if [[ $FAILED -ne 0 ]]; then
		exit 1
	fi
}

main "$@"
