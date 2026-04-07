function {{ alias }} {
	__pr_main suggest
}

function __pr_main {
	param(
			[string]$mode
			)

		$Command = (Get-History -Count 1).CommandLine
		Invoke-Expression (__pr_base $mode $Command)
}

function __pr_base {
	param(
			[string]$mode,
			[string]$Command
			)

	try {
		$env:_PR_PREFIX = (prompt)
		$env:_PR_MODE = $mode
		$env:_PR_LAST_COMMAND = $Command
		# $env:_PR_ALIAS = (Get-Alias | Out-String)
		$env:_PR_SHELL = "pwsh"

		& '{{ binary_path }}'

	} finally {
		$env_PR_PREFIX = $null;
		$env:_PR_MODE = $null;
		$env:_PR_LAST_COMMAND = $null;
		# $env:_PR_ALIAS = $null;
		$env:_PR_SHELL = $null;
	}
}

function __pr_inline {
	$line = $null
		$cursor = $null
		[Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

		$mode = 'inline'
		$command = $line

	$output = __pr_base $mode $command

		if (-not [string]::IsNullOrWhiteSpace($output)) {
			[Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $output)
			[Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($output.Length)
		}
	if ($env:_PR_MODE -eq 'inline') {
		$env:_PR_MODE = $null;
	}
}

Set-PSReadLineKeyHandler -Chord Ctrl+x,Ctrl+x -ScriptBlock { __pr_inline }

# Uncomment this block to enable command not found hook
# It's not very useful as we can't retrieve arguments,

{%- if cnf %}
# function __pr_invoke {
# 	try {
# 		&'{{ binary_path }}' | Invoke-Expression;
# 	} finally {
# 		$env:_PR_MODE = $env:null;
# 		$env:_PR_LAST_COMMAND = $env:null;
# 		$env:_PR_SHELL = $env:null;
# 	}
# }

# $ExecutionContext.InvokeCommand.CommandNotFoundAction = {
# 	param($commandName, $eventArgs)

# 	$env:_PR_LAST_COMMAND = $commandName -replace '^get-|\.\\','';
# 	$env:_PR_SHELL = 'pwsh';
# 	$env:_PR_MODE = 'cnf';

# 	$eventArgs.Command = (Get-Command __pr_invoke);
# 	$eventArgs.StopSearch = $True;
# }
{% endif %}
