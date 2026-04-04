function {{ alias }} -d "Suggest fixes to the previous command"
	eval $(_PR_LAST_COMMAND="$(builtin history | head -n 1)" _PR_ALIAS="$(alias)" _PR_SHELL="fish" "{{ binary_path }}")
end

function __pr_inline
	set input (commandline)

	set output $(_PR_MODE="inline" _PR_LAST_COMMAND="$input" _PR_ALIAS="$(alias)" _PR_SHELL="fish" "{{ binary_path }}")

	if test -n "$output"
		commandline --replace "$output"
	end
end

for mode in default insert
	bind -M $mode \cx\cx __pr_inline
end

{%if cnf %}
if status is-interactive
	function fish_command_not_found --on-event fish_command_not_found
		eval $(_PR_LAST_COMMAND="$argv" _PR_ALIAS="$(alias)" _PR_SHELL="fish" _PR_MODE="cnf" "{{ binary_path }}")
	end
end
{% endif %}
