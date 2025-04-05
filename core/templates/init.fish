function {{ alias }} -d "Suggest fixes to the previous command"
	eval $(_PR_LAST_COMMAND="$(history | head -n 1)" _PR_ALIAS="$(alias)" _PR_SHELL="fish" "{{ binary_path }}")
end

{%if cnf %}
function fish_command_not_found --on-event fish_command_not_found
	eval $(_PR_LAST_COMMAND="$argv" _PR_ALIAS="$(alias)" _PR_SHELL="fish" _PR_MODE="cnf" "{{ binary_path }}")
end
{% endif %}
