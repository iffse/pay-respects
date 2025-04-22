# Roadmap

## Shell Integrations

- ✅: Working
- ⚠️: Partial working
- ❌: Not implemented / Cannot implement

| Feature           | Bash | Zsh | Fish | Nush | Pwsh   |
|-------------------|------|-----|------|------|--------|
| Get other alias   | ✅   | ✅  | ✅   | ✅   | ❌     |
| Command not found | ✅   | ✅  | ✅   | ❌   | ⚠️ [^1] |

[^1]: Cannot retrieve arguments.

## Features Pending Implementation

- Chained commands (e.g. `echo "1\n2" | grepp "2"`)
