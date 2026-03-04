# Phase 4: `download` 指令开发计划（Agent-Native 版）

## 背景与目标

当前项目已完成查询类能力（`search/show/episodes/trending/recent/categories/stats`），缺少“动作型”能力。  
本阶段新增 `podcast download`，用于按单集下载音频文件，同时将接口优化为更易被 AI Agent 编排和脚本消费：

- CLI 参数定义在 `src/cli.rs`
- 命令实现放在 `src/commands/`
- 统一错误类型 `PodcastCliError`
- 输出兼顾人类可读与机器可读，并且支持最小输出模式

---

## 0. 当前设计不足（需要修正）

1. `stdout` 语义不够稳定  
   现有方案以 `table/json` 为主，对 shell/agent 的 `$(...)` 捕获不友好，缺少“只输出路径一行文本”的能力。

2. 缺少“仅探测不下载”能力  
   Agent 常需要先拿到 `enclosure_url/content_type/文件名` 再决策（存储策略、过滤规则、任务分发），当前方案没有 `dry-run`。

3. 进度只有人类可读文本  
   纯文本进度条难以被程序消费，无法稳定做重试策略、可视化监控或并发任务聚合。

4. 退出码契约不明确  
   若仅依赖错误文案，Agent 很难稳定区分“参数错误 / 元数据错误 / 网络错误 / 磁盘错误”。

5. 缺少最小结构化成功输出  
   Agent 往往只需要 `path + size`，当前输出冗余字段较多，解析成本高。

---

## 1. Agent-Native 功能需求

### 1.1 核心能力

1. 用户可通过 `episode-id` 下载单集音频，下载过程流式写盘。
2. 提供 `--dry-run` 仅获取元数据，不进行下载写盘。
3. 提供 `--path-only`（或别名 `--quiet`）输出一行文件路径。
4. 提供 `--progress-json` 输出结构化进度事件（建议 NDJSON，输出到 `stderr`）。
5. 提供 `--minimal` 输出最小 JSON（成功下载时至少包含 `path` 与 `size`）。
6. 保证退出码明确：成功 `0`，失败非 `0`。

### 1.2 CLI 设计（MVP）

```bash
podcast download <episode-id> [OPTIONS]
```

建议选项：

- `--dest <path>`: 目标路径。可为目录或完整文件路径；默认当前目录。
- `--filename <name>`: 自定义文件名（仅当 `--dest` 为目录时生效）。
- `--overwrite`: 目标文件存在时覆盖（默认不覆盖）。
- `--resume`: 若存在 `.part` 临时文件则尝试断点续传。
- `--timeout <seconds>`: 下载超时（默认 `120`）。
- `--no-progress`: 关闭进度显示（文本或 JSON）。
- `--progress-json`: 将进度以 NDJSON 输出到 `stderr`。
- `--dry-run`: 仅探测元数据，不下载。
- `--path-only`: 仅输出目标文件路径到 `stdout`（单行）。
- `--quiet`: `--path-only` 的可见别名（同义行为）。
- `--minimal`: 输出最小 JSON（面向 agent 快速解析）。
- `--output <json|table>`: 默认结果格式（保留现有规范）。

### 1.3 输出模式与冲突规则

为避免歧义，输出模式采用“显式冲突”而非隐式覆盖：

- `--path-only/--quiet` 与 `--minimal`、`--output` 互斥。
- `--minimal` 与 `--output` 互斥。
- `--progress-json` 与 `--no-progress` 互斥。
- `--dry-run` 与 `--resume` 互斥（无写盘，不存在续传语义）。
- `--dry-run` 与 `--overwrite` 互斥（无写盘，不存在覆盖语义）。

默认行为：

- `stdout`: `table`（或 `--output json` 指定 JSON）
- `stderr`: 人类可读进度（`--progress-json` 时改为结构化 NDJSON）

### 1.4 输出契约（关键）

1. `--path-only` / `--quiet`  
   `stdout` 仅一行绝对或规范化路径，无前后缀文案，无额外空行。

2. `--dry-run`（不写盘）  
   返回可序列化元数据，至少包含：
   - `enclosure_url`
   - `content_type`（若可获知）
   - `filename`（推断后的最终文件名）
   - `path`（最终目标路径，仅计划值）

3. `--minimal`（下载成功）  
   最小 JSON：

```json
{"path":"/abs/path/episode.mp3","size":123456}
```

4. `--minimal --dry-run`  
   最小 JSON（元数据模式）：

```json
{"path":"/abs/path/episode.mp3","filename":"episode.mp3","enclosure_url":"https://...","content_type":"audio/mpeg","dry_run":true}
```

5. `--progress-json`（`stderr` NDJSON）  
   事件模型建议：
   - `start`: episode_id, url, path, resumed
   - `progress`: downloaded_bytes, total_bytes, percent, bytes_per_sec
   - `finish`: path, size, elapsed_ms
   - `error`: code, message

---

## 2. 技术方案

### 2.1 HTTP 与元数据获取

- 元数据通过 `PodcastIndexClient + get_episode_by_id` 获取。
- 下载使用独立 `reqwest::Client`（无需 Podcast Index 认证头）。
- `--dry-run` 时不启动流式下载，只做以下动作：
  1. 获取 episode 元数据；
  2. 解析并校验 `enclosure_url`；
  3. 推断文件名与目标路径；
  4. 尝试通过 `HEAD`（失败则回退轻量 `GET`）获取 `Content-Type`。

依赖建议：

- `reqwest` 启用 `stream` feature。
- 使用 `futures-util` 消费 `bytes_stream()`。

### 2.2 路径与扩展名推断

下载功能不转码，仅保存原始字节。扩展名推断优先级：

1. HTTP `Content-Type`
2. episode 的 `enclosureType`
3. `enclosureUrl` 路径扩展名
4. 回退 `.bin`

支持映射：

- `audio/mpeg` -> `.mp3`
- `audio/mp4` / `audio/x-m4a` -> `.m4a`
- `audio/aac` -> `.aac`
- `audio/ogg` -> `.ogg`
- `audio/opus` -> `.opus`
- `audio/wav` / `audio/x-wav` -> `.wav`
- `audio/flac` -> `.flac`
- `video/mp4` -> `.mp4`

### 2.3 流式下载与续传

1. 目标文件采用 `.part` 临时文件写入。
2. 正常下载：分块写盘，完成后 `flush + sync_data + rename`。
3. `--resume`：读取 `.part` 大小并发起 Range 请求。
4. 服务端返回 `206` 则追加写入；返回 `200` 则回退全量下载。
5. 中断时保留 `.part` 供下次续传。

### 2.4 结构化进度输出

- 进度统一输出到 `stderr`，确保 `stdout` 可被稳定管道消费。
- 文本进度与 JSON 进度共用一套内部进度事件，避免双实现漂移。
- `--progress-json` 采用“每行一个 JSON 对象”格式，便于 agent 流式解析。

### 2.5 退出码约定

建议在 `main` 入口统一映射错误类型到退出码：

- `0`: 成功（含 `--dry-run` 成功）
- `2`: 参数/校验错误（含互斥参数、episode id 非法、URL 非法）
- `3`: 配置或认证错误（缺失 key/secret）
- `4`: 元数据错误（episode 不存在、无 enclosure）
- `5`: HTTP/网络错误（超时、连接失败、非预期状态码）
- `6`: 文件系统错误（权限不足、写盘失败、重命名失败）
- `1`: 其他未分类错误（兜底）

---

## 3. 实现步骤（任务拆分）

### Task 1: CLI 与参数约束建模

**修改文件**

- `src/cli.rs`
- `src/commands/mod.rs`

**工作内容**

1. 在 `Commands` 枚举新增 `Download(DownloadArgs)`。
2. 为 `DownloadArgs` 增加新选项：`dry_run/path_only(min alias: quiet)/minimal/progress_json`。
3. 使用 clap 互斥组或 `conflicts_with_all` 明确冲突关系。
4. 在 dispatch 中接入 `download::run(args, manager).await`。
5. `src/commands/mod.rs` 导出 `pub mod download;`。

### Task 2: 结果模型与输出层

**修改文件**

- `src/commands/download.rs`（新建）
- `src/output/table.rs`
- `src/output/json.rs`（若已有统一 JSON helper 则复用）

**工作内容**

1. 定义 `DownloadResult` 与 `DryRunResult`。
2. 定义最小输出结构 `MinimalDownloadResult`。
3. 实现 `stdout` 输出分派：
   - path-only
   - minimal JSON
   - full JSON
   - table
4. 确保 `path-only` 真正只输出一行路径。

### Task 3: 元数据探测与路径决策

**修改文件**

- `src/commands/download.rs`

**工作内容**

1. 复用 `get_episode_by_id` 获取 `Episode`。
2. 校验并提取 `enclosure_url`（仅允许 http/https）。
3. 实现目标路径决策（目录/文件/默认路径）。
4. 实现文件名清理与扩展名推断。
5. 实现 `--dry-run` 早返回（不触发下载写盘）。

### Task 4: 下载执行与进度事件

**修改文件**

- `src/commands/download.rs`
- `Cargo.toml`

**工作内容**

1. 配置下载 client（timeout、redirect、UA）。
2. 实现 `.part` 写入、续传和原子重命名。
3. 抽象统一进度事件并实现两种渲染器：
   - 文本渲染器（默认）
   - JSON NDJSON 渲染器（`--progress-json`）
4. `--no-progress` 时关闭所有进度输出。

### Task 5: 退出码映射

**修改文件**

- `src/main.rs`
- `src/error.rs`（若需扩展错误类别）

**工作内容**

1. 建立 `PodcastCliError -> exit code` 映射函数。
2. 确保 clap 参数错误和业务校验错误返回非 0。
3. 为关键错误路径补充回归测试。

### Task 6: 文档与帮助信息

**修改文件**

- `README.md`
- `ROADMAP.md`

**工作内容**

1. 更新 `download` 用法示例（含 agent 场景）。
2. 增加 `--dry-run`、`--path-only`、`--minimal`、`--progress-json` 示例。
3. 明确退出码语义与脚本建议写法。

---

## 4. 测试计划

### 4.1 CLI 解析与参数冲突测试

新增：`tests/download_cli.rs`

必测用例：

- `parse_download_with_required_episode_id`
- `parse_download_with_path_only_alias_quiet`
- `parse_download_with_dry_run_and_minimal`
- `reject_path_only_with_output`
- `reject_minimal_with_output`
- `reject_progress_json_with_no_progress`
- `reject_dry_run_with_resume`
- `reject_dry_run_with_overwrite`
- `download_rejects_non_numeric_episode_id`
- `download_rejects_zero_timeout`

### 4.2 命令行为测试（无公网依赖）

建议：`tests/download_command.rs`

必测用例：

- 无 credentials 时返回配置/认证错误（退出码 `3`）。
- episode 无 `enclosure_url` 返回元数据错误（退出码 `4`）。
- 非 `http/https` URL 返回参数/校验错误（退出码 `2`）。
- 目标文件存在且未 `--overwrite` 返回文件系统或校验错误（非 0）。
- `--dry-run` 不写任何文件且返回 metadata 字段。
- `--path-only` 的 `stdout` 仅一行路径，无额外文本。
- `--minimal` 返回严格最小 JSON schema。
- `--progress-json` 输出可逐行反序列化的 JSON 事件。

### 4.3 集成下载测试（本地 HTTP 服务）

1. 小文件完整下载成功（bytes 一致，退出码 `0`）。
2. `206 Partial Content` 续传成功。
3. 不支持 Range 自动回退全量下载。
4. 中断后保留 `.part`。
5. `--dry-run` 时不发起实际下载流。

### 4.4 纯函数单元测试

放在 `src/commands/download.rs` 的 `#[cfg(test)]`：

- 文件名清理规则
- 扩展名推断优先级
- 目标路径解析（目录/文件/默认路径）
- 进度事件序列化（NDJSON 字段稳定）

### 4.5 验证命令

```bash
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
```

手工验收建议：

```bash
cargo run -- download <episode-id> --dest ./downloads --output table
cargo run -- download <episode-id> --dest ./downloads --output json
cargo run -- download <episode-id> --path-only
cargo run -- download <episode-id> --minimal
cargo run -- download <episode-id> --dry-run --minimal
cargo run -- download <episode-id> --progress-json --output json
```

---

## 5. Agent 使用示例

```bash
# 1) 只拿路径，便于脚本捕获
saved_path=$(podcast download 123456 --path-only)

# 2) 只探测，不下载
podcast download 123456 --dry-run --output json

# 3) 最小 JSON，减少解析成本
podcast download 123456 --minimal

# 4) 结构化进度（stderr NDJSON）
podcast download 123456 --progress-json --output json 2>progress.log
```
