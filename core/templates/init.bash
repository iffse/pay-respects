alias {{ alias }}='eval $(_PR_LAST_COMMAND="$(fc -ln -1)" _PR_ALIAS="`alias`" _PR_SHELL="bash" "{{ binary_path }}")'

{%- if cnf %}
command_not_found_handle() {
	eval $(_PR_LAST_COMMAND="$@" _PR_ALIAS="`alias`" _PR_SHELL="bash" _PR_MODE="cnf" "{{ binary_path }}")
}
{% endif %}
