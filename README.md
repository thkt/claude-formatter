# claude-formatter

PostToolUse hook for Claude Code. Auto-formats files after Write/Edit using biome.

## Features

- **biome format**: Code formatting (Prettier-compatible)
- **organizeImports**: Auto-sort and group imports
- **Project-local resolution**: Uses `node_modules/.bin/biome` when available

## Installation

### From Source

```bash
cd /tmp
git clone https://github.com/thkt/claude-formatter.git
cd claude-formatter
cargo build --release
cp target/release/formatter ~/.local/bin/
cd .. && rm -rf claude-formatter
```

## Usage

### As Claude Code Hook

Add to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "formatter",
            "timeout": 2000
          }
        ]
      }
    ]
  }
}
```

### With guardrails (recommended)

```json
{
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "guardrails",
            "timeout": 1000
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Write|Edit|MultiEdit",
        "hooks": [
          {
            "type": "command",
            "command": "formatter",
            "timeout": 2000
          }
        ]
      }
    ]
  }
}
```

## Requirements

- [biome](https://biomejs.dev) CLI installed (`brew install biome` or `npm i -g @biomejs/biome`)

Formatting behavior is controlled by your project's `biome.json`.

## Supported File Types

`.ts` `.tsx` `.js` `.jsx` `.mts` `.cts` `.mjs` `.cjs` `.json` `.jsonc` `.css`

## How It Works

1. Receives PostToolUse hook input from Claude Code (stdin JSON)
2. Extracts `file_path` from `tool_input`
3. Runs `biome check --write --linter-enabled=false` (format + organizeImports)
4. Falls back to `biome format --write` if flags unsupported

## Configuration

Create `~/.config/claude-formatter/config.json`:

```json
{
  "enabled": true
}
```

## Exit Codes

| Code | Meaning |
| ---- | ------- |
| 0    | Always  |

The formatter never blocks operations. It silently formats on success and logs errors to stderr.

## License

MIT
