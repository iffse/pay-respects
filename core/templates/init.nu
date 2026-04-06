def --env {{ alias }} [] {
	let command = (history | last).command
	let dir = (__pr_base suggest $command)
	cd $dir
}

def __pr_base [mode: string, command: string] {
	let alias = (help aliases | select name expansion | each ({ |row| $row.name + "=" + $row.expansion }) | str join (char nl))
	let prefix = $(do $env.PROMPT_PREFIX)
	with-env { _PR_MODE: $mode, _PR_PREFIX: $prefix, _PR_LAST_COMMAND: $command, _PR_ALIAS: $alias, _PR_SHELL: nu } {
		`{{ binary_path }}`
	}
}

def __pr_inline [] {
	let input = (commandline)

	let output = (__pr_base inline $input)

	if ($output | is-not-empty) {
		commandline edit --replace $output
	}
}

$env.config.keybindings ++= [
{
name: __pr_inline
	modifier: control
	keycode: char_x
	mode: [emacs, vi_normal, vi_insert]
	event: {
		send: executehostcommand
		cmd: "__pr_inline"
	}
}
]
