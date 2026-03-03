# podcast-cli

A Rust CLI for Podcast Index API with agent-friendly JSON output and human-readable table output.

## Requirements

- Rust toolchain (`cargo`)
- Podcast Index API credentials (`api_key`, `api_secret`)

## Build

```bash
cargo build
```

## Configure Credentials

```bash
cargo run -- config set \
  --api-key "<your_api_key>" \
  --api-secret "<your_api_secret>" \
  --default-output table \
  --max-results 10
```

Show current config:

```bash
cargo run -- config show
```

## Commands

### Search and Details

```bash
cargo run -- search "rust" --limit 5 --output table
cargo run -- show 920666
cargo run -- show --url https://example.com/feed.xml --output json
```

### Episodes and Trending

```bash
cargo run -- episodes 920666 --limit 10
cargo run -- episode 123456 --output json
cargo run -- trending --limit 20
cargo run -- trending --episodes --lang en --output json
```

### Advanced (Phase 3)

```bash
# Latest episodes
cargo run -- recent --limit 10

# Latest feeds since timestamp
cargo run -- recent --feeds --since 1700000000 --output json

# Episodes before timestamp
cargo run -- recent --before 1700000000 --limit 20

# Categories list
cargo run -- categories
cargo run -- categories --output json

# Platform stats
cargo run -- stats
cargo run -- stats --output json
```

## Output Modes

- `--output table`: default, compact human-readable output
- `--output json`: structured output for scripts and AI agents

## Validation Notes

- `--limit` must be in range `1..=100`
- `recent --before` and `recent --since` require integer Unix timestamps
