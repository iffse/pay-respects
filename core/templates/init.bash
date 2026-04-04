alias {{ alias }}='eval $(_PR_LAST_COMMAND="$(fc -ln -1)" _PR_ALIAS="`alias`" _PR_SHELL="bash" "{{ binary_path }}")'

__pr_inline() {
	local input="$READLINE_LINE"

	local output
	output="$(_PR_MODE="inline" _PR_LAST_COMMAND="$input" _PR_ALIAS="`alias`" _PR_SHELL="zsh" "{{ binary_path }}")"

	{% raw %}
	if [[ -n "$output" ]]; then
		READLINE_LINE="$output"
		READLINE_POINT=${#READLINE_LINE}
	fi
	{% endraw %}
}


bind -x '"\C-x\C-x":__pr_inline'

{%- if cnf %}
command_not_found_handle() {
	eval $(_PR_LAST_COMMAND="$@" _PR_ALIAS="`alias`" _PR_SHELL="bash" _PR_MODE="cnf" "{{ binary_path }}")
}
{% endif %}
