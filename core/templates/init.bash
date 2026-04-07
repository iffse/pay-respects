alias {{ alias }}="__pr_main suggest"

__pr_main() {
	eval $(__pr_base "$1" "$(fc -ln -1)")
}

__pr_base() {
	prefix="${PS1@P}"
	_PR_MODE="$1" _PR_PREFIX="$prefix" _PR_LAST_COMMAND="$2" _PR_ALIAS="`alias`" _PR_SHELL="bash" "{{ binary_path }}"
}

__pr_inline() {
	local input="$READLINE_LINE"

	local output
	output=$(__pr_base "inline" "$input")

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
	eval $(__pr_base "cnf" "$@")
}
{% endif %}
