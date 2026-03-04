# podcast-cli

Rust CLI for [Podcast Index API](https://podcastindex-org.github.io/docs-api/), supporting:

- `table` output for human reading
- `json` output for scripts/AI agents

## Prerequisites

- Rust toolchain (`cargo`)
- Podcast Index credentials: `api_key` and `api_secret`
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) (for YouTube features)

## Install

1. Build locally:

```bash
cargo build --release
```

2. Install to `~/.cargo/bin`:

```bash
cargo install --path . --force
```

3. Ensure PATH contains cargo bin:

```bash
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
```

4. Verify:

```bash
podcast-cli --version
podcast-cli --help
```

## Configure API Credentials

```bash
podcast-cli config set \
  --api-key "<your_api_key>" \
  --api-secret "<your_api_secret>" \
  --default-output table \
  --max-results 10
```

Check config:

```bash
podcast-cli config show
```

## Quick Start

```bash
podcast-cli search "rust" --limit 5
podcast-cli show 920666
podcast-cli episodes 920666 --limit 10
podcast-cli trending --limit 5
podcast-cli stats
```

## Command Reference

| Command | Description | Common Options |
|---|---|---|
| `search <term>` | Search podcasts | `--person` `--music` `--limit` `--output` |
| `show <feed-id>` | Show podcast details by id | `--url` `--output` |
| `episodes <feed-id>` | List episodes under a feed | `--limit` `--output` |
| `episode <episode-id>` | Show episode details | `--output` |
| `trending` | Trending podcasts | `--episodes` `--lang` `--limit` `--output` |
| `recent` | Recent updates | `--feeds` `--before` `--since` `--limit` `--output` |
| `categories` | Category list | `--output` |
| `stats` | Platform metrics | `--output` |
| `config set/show/clear` | Manage local config | `--api-key` `--api-secret` `--default-output` `--max-results` |
| `youtube-search <query>` | Search YouTube videos | `--limit` `--channel` `--since` |
| `youtube-subtitles <video-id>` | Download YouTube subtitles | `--lang` `--output` |

## Output Modes

- `--output table`: default, concise table output
- `--output json`: machine-readable output

## Examples

```bash
# Recent episodes
podcast-cli recent --limit 10

# Recent feeds since Unix timestamp
podcast-cli recent --feeds --since 1700000000 --output json

# Categories in JSON
podcast-cli categories --output json

# Stats in JSON
podcast-cli stats --output json

# YouTube search
podcast-cli youtube-search "Sam Altman" --limit 5
podcast-cli youtube-search --channel "Lex Fridman" --since 30d

# YouTube subtitles (JSON output)
podcast-cli youtube-subtitles 5MWT_doo68k --lang en --output json

# YouTube subtitles (SRT format)
podcast-cli youtube-subtitles 5MWT_doo68k --lang en --output srt
```

## Validation Rules

- `--limit` range: `1..=100`
- `recent --before` and `recent --since` must be integer Unix timestamps

## Troubleshooting

1. `command not found: podcast-cli`

```bash
source ~/.zshrc
echo $PATH | tr ':' '\n' | rg '.cargo/bin'
```

2. `Configuration error: api_key is not configured`

```bash
podcast-cli config show
podcast-cli config set --api-key "<key>" --api-secret "<secret>"
```

3. Network/API timeout or DNS issue

```bash
curl -4 -v --connect-timeout 10 https://api.podcastindex.org
```
