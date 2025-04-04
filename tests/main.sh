#!/usr/bin/bash

export PR=$(realpath ../target/debug/pay-respects)
export _PR_SHELL="bash"
export _PR_LIB=""
export _PR_MODE="echo"
export TMP=$(mktemp -d)

SUCESS=0
FAILED=0
export green='\033[0;32m'
export red='\033[0;31m'
export reset='\033[0m'

run_test() {
	export _PR_LAST_COMMAND=$2
	export _PR_ERROR_MSG=$3
	export _PR_SHELL=$4
	result=$($PR 2>/dev/null)
	if [[ $result == *"$expect"* ]]; then
		echo -e "${green}[Passed]${reset}: $1"
		return 0
	else
		echo -e "${red}[Failed]${reset}: $1"
		echo -e "\texpected: $expect"
		echo -e "\tgot: $result"
		return 1
	fi
}

run_case() {
	source $1
	eval $pre
	run_test "$case" "$command" "$error" "$expect"

	if [[ $? == 0 ]]; then
		SUCESS=$((SUCESS + 1))
	else
		FAILED=$((FAILED + 1))
	fi
}

main() {
	echo "Starting suggestion tests..."
	echo "-----------------------------------------"
	cd cases
	WORKDIR=$(pwd)
	for case in *; do
		cd $WORKDIR
		run_case $case
	done
	
	echo "-----------------------------------------"
	echo -en "${green}Success${reset}: $SUCESS\t"
	echo -en "${red}Failed${reset}: $FAILED\t"
	echo -e "Total: $((SUCESS + FAILED))"

	rm -rf $TMP
	if [[ $FAILED -ne 0 ]]; then
		exit 1
	fi
}

main "$@"
