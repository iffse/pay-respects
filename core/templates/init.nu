def --env {{ alias }} [] {
	let dir = (with-env { _PR_LAST_COMMAND: (history | last).command, _PR_ALIAS: (help aliases | select name expansion | each ({ |row| $row.name + "=" + $row.expansion }) | str join (char nl)), _PR_SHELL: nu } { `{{ binary_path }}` })
	cd $dir
}
