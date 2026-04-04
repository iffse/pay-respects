alias {{ alias }}='eval $(_PR_LAST_COMMAND="$(fc -ln -1)" _PR_ALIAS="`alias`" _PR_SHELL="zsh" "{{ binary_path }}")'

function __pr_inline() {
	local input="$BUFFER"
	local output
	export _PR_MODE="inline"

	output="$(_PR_LAST_COMMAND="$input" _PR_ALIAS="`alias`" _PR_SHELL="zsh" "{{ binary_path }}")"

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
	eval $(_PR_LAST_COMMAND="$@" _PR_SHELL="zsh" _PR_ALIAS="`alias`" _PR_MODE="cnf" "{{ binary_path }}")
}
{% endif %}
