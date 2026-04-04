def --env {{ alias }} [] {
	let dir = (with-env { _PR_LAST_COMMAND: (history | last).command, _PR_ALIAS: (help aliases | select name expansion | each ({ |row| $row.name + "=" + $row.expansion }) | str join (char nl)), _PR_SHELL: nu } { `{{ binary_path }}` })
	cd $dir
}

def __pr_inline [] {
	let input = (commandline)

	let output = (with-env { _PR_MODE = "inline", _PR_LAST_COMMAND: $input, _PR_ALIAS: (help aliases | select name expansion | each ({ |row| $row.name + "=" + $row.expansion }) | str join (char nl)), _PR_SHELL: nu } { `{{ binary_path }}` })

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
