# Phase 5: `youtube-subtitles` 功能开发计划（Agent-Native 版）

## 背景与目标

该功能用于 AI 播客周报流程中的“内容采集”环节：从 YouTube 视频提取字幕，输出给后续 LLM 做摘要、主题提取、观点归纳与结构化分析。

本阶段新增命令：

```bash
podcast youtube-subtitles <video-id> [--lang <en|zh|...>] [--output <json|text|srt>]
```

核心目标：

1. 支持通过 `video-id` 下载字幕
2. 支持语言选择（`--lang`）
3. 支持三种输出格式（`json/text/srt`）
4. 输出稳定、结构化、易于 Agent/脚本消费
5. 保持与现有项目一致的错误处理与 CLI 风格

---

## 方案对比（先设计再实现）

### 方案 A：完全依赖 `yt-dlp` 下载字幕文件并直接输出

- 优点：实现快、依赖少、与 YouTube 兼容性高
- 缺点：`json/text/srt` 三种统一输出契约难保证；结构化字段不稳定，后续 LLM 接口不易固定

### 方案 B（推荐）：`yt-dlp` 负责“解析字幕轨道”，Rust 负责“标准化输出”

- 优点：
  - 满足“优先使用 yt-dlp”
  - 输出契约完全由本项目控制（Agent-Native）
  - 可统一错误分类与退出码
  - 便于未来增加字段（speaker/chunk/token_count 等）
- 缺点：实现复杂度高于方案 A（需做轨道选择与格式归一化）

### 方案 C：替换为第三方字幕 API / 非 `yt-dlp` 方案

- 优点：可规避本地工具依赖
- 缺点：额外成本、稳定性与合规风险更高，不符合“优先 yt-dlp”要求

结论：采用方案 B。

---

## 与现有代码的一致性约束

1. 命令定义仍在 `src/cli.rs`，由 `src/commands/mod.rs` 分发
2. 命令实现放在 `src/commands/youtube_subtitles.rs`
3. 统一使用 `PodcastCliError` / `Result<T>` 返回错误
4. `stdout` 只放业务结果，错误输出走 `stderr`（保持 `main.rs` 现有行为）
5. 测试风格对齐现有 `tests/*_cli.rs`（参数解析/冲突关系优先）

---

## 命令与输出契约设计

## 1) CLI 设计

新增子命令：

```bash
podcast youtube-subtitles <video-id> [OPTIONS]
```

参数建议：

- `video-id`：必填，YouTube 视频 ID（建议校验长度 11 且字符集合法）
- `--lang <code>`：可选，默认 `en`
- `--output <json|text|srt>`：可选，默认 `json`

说明：

- 该命令的 `--output` 不复用全局 `OutputArg(json|table)`，需要新增独立枚举，避免与现有 `table` 语义冲突。

## 2) Agent-Native JSON 输出（默认）

默认输出建议：

```json
{
  "video_id": "dQw4w9WgXcQ",
  "language": "en",
  "source": "manual",
  "title": "Video title",
  "duration_sec": 212,
  "segments": [
    {"index": 1, "start_ms": 0, "end_ms": 1530, "text": "hello world"}
  ],
  "text": "hello world ...",
  "segment_count": 120
}
```

约束：

- 字段名稳定（snake_case）
- `segments` 为主数据，`text` 为便捷聚合文本
- 允许空 `segments`，但必须有明确错误或空数据语义

## 3) text / srt 输出

- `text`：仅输出纯文本字幕（按时间顺序拼接，适合 LLM 输入）
- `srt`：输出标准 SRT 文本（`1\n00:00:00,000 --> ...`）

---

## 技术设计

## 1) 外部工具调用（yt-dlp）

调用方式（示意）：

```bash
yt-dlp --skip-download --no-warnings --dump-single-json "https://www.youtube.com/watch?v=<video-id>"
```

流程：

1. 检查 `yt-dlp` 可执行文件是否存在（`--version`）
2. 获取视频 metadata（含 `subtitles` / `automatic_captions`）
3. 按 `--lang` 选择最优字幕轨道（先人工字幕，再自动字幕）
4. 下载字幕原文（优先结构化格式，如 `json3`；否则 `vtt/srt`）

## 2) 字幕归一化

内部统一结构：

- `SubtitleSegment { index, start_ms, end_ms, text }`
- `YoutubeSubtitlesResult { video_id, language, source, title, duration_sec, segments, text, segment_count }`

归一化优先级：

1. `json3`（结构最完整）
2. `vtt`
3. `srt`

## 3) 错误处理映射

建议错误语义：

- `Validation`：video-id 非法、`--lang` 为空、`--output` 非法
- `Config`：`yt-dlp` 不存在（可提示安装命令）
- `Metadata`：视频无可用字幕 / 指定语言不存在
- `Api`：`yt-dlp` 调用失败、YouTube 请求失败
- `Serialization`：字幕解析失败（json/vtt/srt 格式异常）

---

## 实现任务拆分（可执行）

### Task 1: CLI 接口定义与参数测试

**修改文件**

- `src/cli.rs`
- `tests/youtube_subtitles_cli.rs`（新建）

**工作内容**

1. 新增 `Commands::YoutubeSubtitles(YoutubeSubtitlesArgs)`
2. 新增 `YoutubeSubtitlesArgs` 与 `SubtitleOutputArg`
3. 增加 `video-id` 校验函数（例如长度/字符集）
4. 增加 CLI 解析测试：
   - 默认值（`lang=en`, `output=json`）
   - 显式 `--lang zh --output srt`
   - 非法 video-id 报错

**验收**

- `cargo test --test youtube_subtitles_cli`

### Task 2: 命令分发与骨架实现

**修改文件**

- `src/commands/mod.rs`
- `src/commands/youtube_subtitles.rs`（新建）

**工作内容**

1. 导出 `pub mod youtube_subtitles;`
2. 在 `dispatch` 中接入新命令
3. 新建 `run(args, manager)` 骨架，返回统一 `Result<()>`

**验收**

- `cargo check`

### Task 3: yt-dlp 适配层（元数据获取 + 轨道选择）

**修改文件**

- `src/commands/youtube_subtitles.rs`

**工作内容**

1. 实现 `ensure_yt_dlp_available()`
2. 实现 `fetch_video_metadata(video_id)`（调用 `yt-dlp --dump-single-json`）
3. 定义最小 metadata 反序列化结构
4. 实现 `select_track_by_lang(lang)`：
   - 先匹配人工字幕
   - 再匹配自动字幕
   - 支持 `en` 匹配 `en-US` 的前缀策略

**验收**

- 增加模块内单元测试：轨道选择逻辑、语言匹配逻辑

### Task 4: 字幕下载与归一化解析

**修改文件**

- `src/commands/youtube_subtitles.rs`

**工作内容**

1. 下载轨道 URL 内容（`reqwest`）
2. 解析 `json3/vtt/srt` 到统一 `SubtitleSegment`
3. 构建聚合 `text` 与 `segment_count`
4. 对空字幕/异常格式提供明确错误

**验收**

- 增加模块内测试：`json3`、`vtt`、`srt` fixture 到统一 segments

### Task 5: 输出渲染（json/text/srt）

**修改文件**

- `src/commands/youtube_subtitles.rs`

**工作内容**

1. `json`：输出结构化结果（pretty JSON）
2. `text`：输出纯文本
3. `srt`：将统一 segments 反序列化为标准 SRT
4. 确保 `stdout` 仅输出结果内容

**验收**

- 增加模块测试：格式化结果与关键字段断言

### Task 6: 文档与用户引导

**修改文件**

- `README.md`
- `ROADMAP.md`（如需记录 Phase 5 完成状态）

**工作内容**

1. 增加 `youtube-subtitles` 命令说明
2. 增加依赖说明：需安装 `yt-dlp`
3. 增加常见错误排查（`yt-dlp not found` / 指定语言无字幕）

**验收**

- 文档示例命令可直接运行（在有 `yt-dlp` 环境）

---

## 测试与验证计划

开发完成后执行：

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

建议补充一次手动验收：

```bash
podcast youtube-subtitles <video-id> --lang en --output json
podcast youtube-subtitles <video-id> --lang zh --output text
podcast youtube-subtitles <video-id> --lang en --output srt
```

---

## 风险与缓解

1. `yt-dlp` 输出结构可能变化  
缓解：只依赖最小字段；metadata 解析做 `Option` 容错。

2. 字幕格式多样导致解析脆弱  
缓解：内部统一 segment 模型 + 三类格式 fixture 测试。

3. 指定语言不可用  
缓解：返回明确错误，并在错误消息中附带可用语言列表。

4. 外部网络波动影响稳定性  
缓解：请求超时、错误分类、必要重试（后续可增量）。

---

## 里程碑定义

- M1（CLI 就绪）：参数解析 + 分发 + 基础测试通过
- M2（能力可用）：可下载并输出 `json/text/srt`
- M3（Agent-Native）：结构化输出稳定，错误语义清晰，文档齐全
