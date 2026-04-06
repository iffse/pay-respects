alias {{ alias }}='eval $(__pr_main)'

function __pr_main() {
	__pr_base "suggest" "$(fc -ln -1)"
}

function __pr_base() {
	prefix=$(print -P "$PROMPT")
	_PR_MODE="$1" _PR_PREFIX="$prefix" _PR_LAST_COMMAND="$2" _PR_ALIAS="`alias`" _PR_SHELL="zsh" "{{ binary_path }}"
}

function __pr_inline() {
	local input="$BUFFER"
	local output

	output=$(_pr_base "inline" "$input")

	{% raw %}
	if [[ -n "$output" ]]; then
		BUFFER="$output"
		CURSOR=${#BUFFER}
	fi
	{% endraw %}
}

zle -N __pr_inline
bindkey '^X^X' __pr_inline

{%- if cnf %}
command_not_found_handler() {
	eval $(__pr_base "cnf" "$@")
}
{% endif %}
