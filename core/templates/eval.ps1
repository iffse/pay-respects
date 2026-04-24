{%- if !command.is_empty() %}
try { [Microsoft.PowerShell.PSConsoleReadLine]::AddToHistory('{{ command }}') } catch {}
{%- endif %}
{%- if let Some(cd) = self.cd %}
cd {{ cd }}
{%- endif %}
