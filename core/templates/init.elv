use str

var _pr_base = {|mode last_command|
	tmp E:_PR_MODE = $mode
	tmp E:_PR_LAST_COMMAND = $last_command
	tmp E:_PR_SHELL = '{{ shell }}'
	tmp E:_PR_SHELL_TYPE = '{{ shell_type }}'
	{%- if let Some(prefix) = self.prefix %}
	tmp E:_PR_PREFIX = '{{ prefix }}'
	{%- endif %}
	'{{ binary_path }}'
}

var _pr_suggest = {
	use store
	var last_command = (edit:command-history &cmd-only &newest-first | drop 1 | take 1)
	var output = ''
	try {
		set output = ($_pr_base suggest $last_command | slurp | str:trim-space)
	} catch {
	}
	if (not-eq $output '') {
		eval $output
	}
}

var _pr_inline = {
	var input = $edit:current-command
	var output = ''
	try {
		set output = ($_pr_base inline $input | slurp | str:trim-space)
	} catch {
	}
	if (not-eq $output '') {
		edit:replace-input $output
	}
}

# eval runs in a temporary namespace, so export the alias explicitly
edit:add-var {{ alias }}~ $_pr_suggest
set edit:insert:binding[Ctrl-X] = $_pr_inline
