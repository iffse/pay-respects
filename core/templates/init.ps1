function {{ alias }} {
	try {
		$env:_PR_PREFIX = (prompt)
		# fetch command and error from session history only when not in cnf mode
		if ($env:_PR_MODE -ne 'cnf') {
			$env:_PR_LAST_COMMAND = (Get-History | Select-Object -Last 1 | ForEach-Object {$_.CommandLine});
			# Probabily slower than screen capture
			# Uncomment if you want to get error message from powershell
			# if ($PSVersionTable.PSVersion.Major -ge 7) {
			# 	$err = Get-Error;
			# 	if ($env:_PR_LAST_COMMAND -eq $err.InvocationInfo.Line) {
			# 		$env:_PR_ERROR_MSG = $err.Exception.Message
			# 	}
			# }
		}
		$env:_PR_SHELL = 'pwsh';
		&'{{ binary_path }}' | Invoke-Expression;
	}
	finally {
# restore mode from cnf
		if ($env:_PR_MODE -eq 'cnf') {
			$env:_PR_MODE = $env:_PR_PWSH_ORIGIN_MODE;
			$env:_PR_PWSH_ORIGIN_MODE = $null;
		}
	}
}

function Invoke-PRInline {
	$line = $null
		$cursor = $null
		[Microsoft.PowerShell.PSConsoleReadLine]::GetBufferState([ref]$line, [ref]$cursor)

	$env:_PR_MODE = 'inline';
	$env:_PR_LAST_COMMAND = $line;
	$env:_PR_SHELL = 'pwsh';

	$output = & '{{ binary_path }}'

		if (-not [string]::IsNullOrWhiteSpace($output)) {
			[Microsoft.PowerShell.PSConsoleReadLine]::Replace(0, $line.Length, $output)
			[Microsoft.PowerShell.PSConsoleReadLine]::SetCursorPosition($output.Length)
		}
	if ($env:_PR_MODE -eq 'inline') {
		$env:_PR_MODE = $null;
	}
}

Set-PSReadLineKeyHandler -Chord Ctrl+x,Ctrl+x -ScriptBlock { Invoke-PRInline }

{%- if cnf %}
# Uncomment this block to enable command not found hook
# It's not useful as we can't retrieve arguments,
# and it seems to be a recursion bug

# $ExecutionContext.InvokeCommand.CommandNotFoundAction =
# {
# 	param(
# 		[string]
# 		$commandName,
# 		[System.Management.Automation.CommandLookupEventArgs]
# 		$eventArgs
# 	)
# 	# powershell does not support run command with specific environment variables
# 	# but you must set global variables. so we are memorizing the current mode and the alias function will reset it later.
# 	$env:_PR_PWSH_ORIGIN_MODE=$env:_PR_MODE;
# 	$env:_PR_MODE='cnf';
# 	# powershell may search command with prefix 'get-' or '.\' first when this hook is hit, strip them
# 	$env:_PR_LAST_COMMAND=$commandName -replace '^get-|\.\\','';
# 	$eventArgs.Command = (Get-Command {{ alias }});
# 	$eventArgs.StopSearch = $True;
# }
{% endif %}
