# podcast-cli

[English](README.md) | [中文](README_zh.md)

Rust CLI for [Podcast Index API](https://podcastindex-org.github.io/docs-api/), supporting:

- `table` output for human reading
- `json` output for scripts/AI agents
- YouTube video search and subtitle download
- Podcast episode download and transcription

## Prerequisites

- Rust toolchain (`cargo`)
- Podcast Index credentials: `api_key` and `api_secret`
- [yt-dlp](https://github.com/yt-dlp/yt-dlp) (for YouTube features)
- [ffmpeg](https://ffmpeg.org/) (for audio processing)
- [OpenAI Whisper](https://github.com/openai/whisper) (for transcription)

## Install

```bash
cargo install --git https://github.com/the-waste-land/podcast-cli.git --tag v0.2.1
```

Or build locally:

```bash
cargo build --release
cargo install --path . --force
```

## Configure API Credentials

```bash
podcast-cli config set \
  --api-key "<your_api_key>" \
  --api-secret "<your_api_secret>" \
  --default-output table \
  --max-results 10
```

## Proxy Configuration

If you need a proxy to access the Podcast Index API:

```bash
export HTTP_PROXY=http://127.0.0.1:7890
export HTTPS_PROXY=http://127.0.0.1:7890
# or ALL_PROXY for all protocols
```

Supported env vars: `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY` (and lowercase variants)

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
| `download <episode-id>` | Download episode audio | `--dest` |
| `transcribe <audio-file>` | Transcribe audio with Whisper | `--model` `--language` |
| `trending` | Trending podcasts | `--episodes` `--lang` `--limit` `--output` |
| `recent` | Recent updates | `--feeds` `--before` `--since` `--limit` `--output` |
| `categories` | Category list | `--output` |
| `stats` | Platform metrics | `--output` |
| `config set/show/clear` | Manage local config | `--api-key` `--api-secret` `--default-output` `--max-results` |
| `youtube-search <query>` | Search YouTube videos | `--limit` `--channel` `--since` `--with-meta` `--meta-concurrency` `--meta-timeout` `--json-envelope` |
| `youtube-meta <video-id>` | Fetch YouTube metadata for a single video | `--output <json&#124;table>` |
| `youtube-subtitles <video-id>` | Download YouTube subtitles | `--lang` `--output` |

## Output Modes

- `--output table`: default, concise table output
- `--output json`: machine-readable output

## Examples

```bash
# Search podcasts
podcast-cli search "Sam Altman" --limit 5

# Show podcast details
podcast-cli show 6023552

# List episodes
podcast-cli episodes 6023552 --limit 10

# Download episode audio
podcast-cli download 51062882089 --dest ./episode.mp3

# Transcribe audio (requires Whisper)
podcast-cli transcribe ./episode.mp3 --language en

# YouTube search
podcast-cli youtube-search "Sam Altman" --limit 5
podcast-cli youtube-search --channel "Lex Fridman" --since 30d
podcast-cli youtube-search "Sam Altman" --limit 10 --with-meta
podcast-cli youtube-search "Sam Altman" --with-meta --meta-concurrency 4 --meta-timeout 20
podcast-cli youtube-search "Sam Altman" --with-meta --json-envelope

# YouTube single-video metadata
podcast-cli youtube-meta 5MWT_doo68k
podcast-cli youtube-meta 5MWT_doo68k --output table

# YouTube subtitles
podcast-cli youtube-subtitles 5MWT_doo68k --lang en --output json
podcast-cli youtube-subtitles 5MWT_doo68k --lang en --output srt

# Trending and recent
podcast-cli trending --limit 10
podcast-cli recent --limit 10
podcast-cli recent --feeds --since 1700000000 --output json

# Stats
podcast-cli stats --output json

# Categories
podcast-cli categories --output json
```

## Validation Rules

- `--limit` range: `1..=100`
- `recent --before` and `recent --since` must be integer Unix timestamps
- `youtube-search --meta-concurrency` requires `--with-meta`; range: `1..=16`
- `youtube-search --meta-timeout` requires `--with-meta`; range: `1..=120` seconds

## YouTube Metadata Fields

`youtube-meta` returns a stable JSON object with explicit nullable fields:

- `video_id`, `title`, `channel`, `url`
- `duration`, `upload_date`, `timestamp`
- `view_count`, `like_count`, `comment_count`
- `availability`

For `youtube-search --with-meta`, the existing result shape is preserved and these fields are appended:
`timestamp`, `view_count`, `like_count`, `comment_count`, `availability`, `meta_status`.
If metadata fetch fails for one item, those appended fields are returned as `null` for that item and `meta_status` is set to `failed` or `timeout`.

`meta_status` values:

- `ok`: metadata fetched successfully
- `failed`: metadata fetch failed (non-timeout error)
- `timeout`: metadata fetch timed out
- `skipped`: fallback status when metadata is skipped

## YouTube Search JSON Shapes

Default mode returns a JSON array (backward-compatible):

```json
[
  {
    "video_id": "...",
    "title": "...",
    "channel": "...",
    "duration": 3600,
    "upload_date": "2026-03-05",
    "url": "https://www.youtube.com/watch?v=..."
  }
]
```

Envelope mode (`--json-envelope`) returns an object:

```json
{
  "query": "AI interview",
  "items": [],
  "meta": {
    "searched": 20,
    "with_meta": true,
    "meta_success": 18,
    "meta_failed": 1,
    "meta_timeout": 1
  }
}
```

`upload_date` normalization in YouTube outputs:

- empty string / whitespace / `NA` / `null` => `null`
- `YYYYMMDD` => `YYYY-MM-DD`

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

4. `yt-dlp` errors for YouTube commands

```bash
yt-dlp --version
yt-dlp --skip-download --dump-single-json "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
```
