# Roadmap

## Shell Integrations

- ✅: Working
- ⚠️: Partial working
- ❌: Not implemented / Cannot implement

| Feature           | Bash | Zsh | Fish | Dash | Xonsh   | Elvish | Nush | Pwsh   |
|-------------------|------|-----|------|------|---------|--------|------|--------|
| Inline mode       | ✅   | ✅  | ✅   | ❌   | ⚠️ [^2] | ✅     | ✅   | ✅     |
| Get other alias   | ✅   | ✅  | ✅   | ✅   | ✅      | ❌     | ✅   | ❌     |
| Command not found | ✅   | ✅  | ✅   | ❌   | ⚠️ [^3] | ❌     | ❌   | ⚠️ [^1] |

[^1]: Cannot retrieve arguments.
[^2]: Requires prompt_toolkit.
[^3]: Requires Xonsh 0.25 or newer.

## Features Pending Implementation

- Chained commands (e.g. `echo "1\n2" | grepp "2"`)
