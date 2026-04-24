def --env {{ alias }} [] {
	__pr_main suggest
}

def --env __pr_main [mode: string] {
	let command = (history | last).command
	let output = (__pr_base $mode $command)
	if ($output | str trim | is-empty) { return }

	let wrapped = ('[' + ($output | str replace -r '}\s*{' '},{') + ']')
	let data = (try { $wrapped | from json } catch { null })
	if $data == null { return }

	for d in $data {
		if ($d.command != "") {
			if $env.config.history.file_format == "plaintext" {
				$"($d.command)\n" | save --append $nu.history-path
			} else {
				[$d.command] | history import
			}
		}
		if ($d.cd != "") {
			cd $d.cd
		}
	}
}

def __pr_base [mode: string, command: string] {
	let alias = (help aliases | select name expansion | each ({ |row| $row.name + "=" + $row.expansion }) | str join (char nl))
	let prefix = if ($env.PROMPT_INDICATOR | is-not-empty) { $env.PROMPT_INDICATOR } else { do $env.PROMPT_COMMAND }
	with-env { _PR_MODE: $mode, _PR_PREFIX: $prefix, _PR_LAST_COMMAND: $command, _PR_ALIAS: $alias, _PR_SHELL: {{ shell }} } {
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
