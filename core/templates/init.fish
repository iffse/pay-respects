function {{ alias }} -d "Suggest fixes to the previous command"
	eval $(__pr_base "suggest" (builtin history | head -n 1))
end

function __pr_base -a mode last_command
	set prefix (set -q SHELL_PROMPT_SUFFIX; and echo $SHELL_PROMPT_SUFFIX; or fish_prompt)
	_PR_MODE="$mode" _PR_PREFIX="$prefix" _PR_LAST_COMMAND="$last_command" _PR_ALIAS="$(alias)" _PR_SHELL="fish" "{{ binary_path }}"
end

function __pr_inline
	set input (commandline)

	set output __pr_base "inline" "$input"

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
		eval $(__pr_base "cnf" "$argv")
	end
end
{% endif %}
