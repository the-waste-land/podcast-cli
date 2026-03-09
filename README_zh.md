# podcast-cli

[English](README.md) | [中文](README_zh.md)

基于 Rust 编写的 [Podcast Index API](https://podcastindex-org.github.io/docs-api/) 命令行工具，支持：

- 适合人类阅读的 `table` 表格输出
- 适合脚本和 AI 代理的 `json` 输出
- YouTube 视频搜索与字幕下载
- 播客单集下载与转录（语音识别）

## 依赖条件

- Rust 工具链 (`cargo`)
- Podcast Index 凭据：`api_key` 和 `api_secret`
- [yt-dlp](https://github.com/yt-dlp/yt-dlp)（用于 YouTube 相关功能）
- [ffmpeg](https://ffmpeg.org/)（用于音频处理）
- [OpenAI Whisper](https://github.com/openai/whisper)（用于音频转录）

## 安装

```bash
cargo install --git https://github.com/the-waste-land/podcast-cli.git --tag v0.2.1
```

或者在本地构建：

```bash
cargo build --release
cargo install --path . --force
```

## 配置 API 凭据

```bash
podcast-cli config set \
  --api-key "<您的_api_key>" \
  --api-secret "<您的_api_secret>" \
  --default-output table \
  --max-results 10
```

## 代理配置

如果您需要通过代理访问 Podcast Index API：

```bash
export HTTP_PROXY=http://127.0.0.1:7890
export HTTPS_PROXY=http://127.0.0.1:7890
# 或者使用 ALL_PROXY 代理所有协议
```

支持的环境变量：`HTTP_PROXY`、`HTTPS_PROXY`、`ALL_PROXY`（及其小写变体）

## 快速开始

```bash
podcast-cli search "rust" --limit 5
podcast-cli show 920666
podcast-cli episodes 920666 --limit 10
podcast-cli trending --limit 5
podcast-cli stats
```

## 命令参考

| 命令 | 描述 | 常用选项 |
|---|---|---|
| `search <term>` | 搜索播客 | `--person` `--music` `--limit` `--output` |
| `show <feed-id>` | 通过 id 查看播客详情 | `--url` `--output` |
| `episodes <feed-id>` | 列出某个播客下的单集 | `--limit` `--output` |
| `episode <episode-id>` | 查看单集详情 | `--output` |
| `download <episode-id>` | 下载单集音频 | `--dest` |
| `transcribe <audio-file>` | 使用 Whisper 转录音频 | `--model` `--language` |
| `trending` | 热门播客 | `--episodes` `--lang` `--limit` `--output` |
| `recent` | 最近更新 | `--feeds` `--before` `--since` `--limit` `--output` |
| `categories` | 分类列表 | `--output` |
| `stats` | 平台统计数据 | `--output` |
| `config set/show/clear` | 管理本地配置 | `--api-key` `--api-secret` `--default-output` `--max-results` |
| `youtube-search <query>` | 搜索 YouTube 视频 | `--limit` `--channel` `--since` `--with-meta` `--meta-concurrency` `--meta-timeout` `--json-envelope` |
| `youtube-meta <video-id>` | 获取单个 YouTube 视频的元数据 | `--output <json&#124;table>` |
| `youtube-subtitles <video-id>` | 下载 YouTube 字幕 | `--lang` `--output` |

## 输出模式

- `--output table`: 默认，简洁的表格输出
- `--output json`: 机器可读的 JSON 输出

## 示例

```bash
# 搜索播客
podcast-cli search "Sam Altman" --limit 5

# 查看播客详情
podcast-cli show 6023552

# 列出单集
podcast-cli episodes 6023552 --limit 10

# 下载单集音频
podcast-cli download 51062882089 --dest ./episode.mp3

# 转录音频（需要 Whisper）
podcast-cli transcribe ./episode.mp3 --language en

# YouTube 搜索
podcast-cli youtube-search "Sam Altman" --limit 5
podcast-cli youtube-search --channel "Lex Fridman" --since 30d
podcast-cli youtube-search "Sam Altman" --limit 10 --with-meta
podcast-cli youtube-search "Sam Altman" --with-meta --meta-concurrency 4 --meta-timeout 20
podcast-cli youtube-search "Sam Altman" --with-meta --json-envelope

# 获取 YouTube 单个视频的元数据
podcast-cli youtube-meta 5MWT_doo68k
podcast-cli youtube-meta 5MWT_doo68k --output table

# 下载 YouTube 字幕
podcast-cli youtube-subtitles 5MWT_doo68k --lang en --output json
podcast-cli youtube-subtitles 5MWT_doo68k --lang en --output srt

# 热门与最近更新
podcast-cli trending --limit 10
podcast-cli recent --limit 10
podcast-cli recent --feeds --since 1700000000 --output json

# 统计数据
podcast-cli stats --output json

# 分类
podcast-cli categories --output json
```

## 验证规则

- `--limit` 范围：`1..=100`
- `recent --before` 和 `recent --since` 必须是整数 Unix 时间戳
- `youtube-search --meta-concurrency` 需要搭配 `--with-meta` 使用；范围：`1..=16`
- `youtube-search --meta-timeout` 需要搭配 `--with-meta` 使用；范围：`1..=120` 秒

## YouTube 元数据字段

`youtube-meta` 返回一个稳定的 JSON 对象，包含明确的可为空字段：

- `video_id`, `title`, `channel`, `url`
- `duration`, `upload_date`, `timestamp`
- `view_count`, `like_count`, `comment_count`
- `availability`

对于 `youtube-search --with-meta`，将保留现有的结果结构，并附加以下字段：
`timestamp`、`view_count`、`like_count`、`comment_count`、`availability`、`meta_status`。
如果某个项目的元数据获取失败，则该项目附加的字段将返回 `null`，且 `meta_status` 会被设置为 `failed` 或 `timeout`。

`meta_status` 的可能值：

- `ok`: 成功获取元数据
- `failed`: 元数据获取失败（非超时错误）
- `timeout`: 元数据获取超时
- `skipped`: 元数据被跳过时的后备状态

## YouTube 搜索 JSON 结构

默认模式返回一个 JSON 数组（向后兼容）：

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

信封模式（Envelope mode，`--json-envelope`）返回一个对象：

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

YouTube 输出中的 `upload_date` 格式化：

- 空字符串 / 空白 / `NA` / `null` => `null`
- `YYYYMMDD` => `YYYY-MM-DD`

## 常见问题排查 (Troubleshooting)

1. `command not found: podcast-cli` (找不到命令：podcast-cli)

```bash
source ~/.zshrc
echo $PATH | tr ':' '\n' | rg '.cargo/bin'
```

2. `Configuration error: api_key is not configured` (配置错误：未配置 api_key)

```bash
podcast-cli config show
podcast-cli config set --api-key "<key>" --api-secret "<secret>"
```

3. 网络/API 超时或 DNS 问题

```bash
curl -4 -v --connect-timeout 10 https://api.podcastindex.org
```

4. 执行 YouTube 相关命令时出现 `yt-dlp` 错误

```bash
yt-dlp --version
yt-dlp --skip-download --dump-single-json "https://www.youtube.com/watch?v=dQw4w9WgXcQ"
```
