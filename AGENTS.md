# Repository Guidelines

## Codex + Claude Code 编排工作流

### 推荐流程
```
1. 需求分析 → Codex 制定计划 (plans/phase-X-xxx.md)
       ↓
2. Codex 实现代码
       ↓
3. Claude Code Code Review
       ↓
4. (可选) Codex 修复 Review 发现的问题
       ↓
5. 验证 (cargo check/test/fmt/clippy) + 提交
```

### 启动 Codex (Persist 模式)
```bash
# 使用 sessions_spawn，mode: session，但 Feishu 不支持 thread
# 所以用 run 模式更稳定
sessions_spawn(
  agentId: "codex",
  cwd: "/path/to/project",
  label: "codex-xxx",
  mode: "run",  # 一次性的任务
  runtime: "acp",
  task: "..."
)
```

### 启动 Claude Code Code Review
```bash
exec(
  pty: true,
  command: "claude -p '请审查 xxx.rs 的代码...'",
  timeout: 180
)
```

### 项目目录结构建议
```
podcast-cli/
├── plans/           # Codex 写的计划 (plans/phase-4-download.md)
├── reviews/         # Claude Code 的审核报告
└── ...
```

### 标签约定
| 标签 | 用途 |
|------|------|
| `codex-impl` | Codex 实现代码 |
| `codex-fix` | Codex 修复 Review 问题 |
| `claude-review` | Claude Code 审核 |

---

## Project Structure & Module Organization
- Core Rust code lives in `src/`.
- CLI definitions are in `src/cli.rs`; command handlers are in `src/commands/`.
- API client/types live in `src/api/` with endpoint-specific modules under `src/api/endpoints/`.
- Config and output formatting are split into `src/config/` and `src/output/`.
- Integration-style tests are in `tests/` (for example `tests/trending_cli.rs`, `tests/config_cli.rs`).
- Planning/reference docs are in `README.md` and `ROADMAP.md`.

## Build, Test, and Development Commands
- `cargo build` builds a debug binary.
- `cargo build --release` builds an optimized binary (used for distribution).
- `cargo run -- <args>` runs locally, e.g. `cargo run -- search rust --limit 5`.
- `cargo test` runs the full test suite under `tests/` and unit tests.
- `cargo fmt -- --check` verifies formatting.
- `cargo clippy --all-targets --all-features -- -D warnings` enforces lint cleanliness matching CI.

## Coding Style & Naming Conventions
- Follow standard Rust style with `rustfmt` (4-space indentation, trailing commas where rustfmt applies).
- Use `snake_case` for functions/modules/files and `PascalCase` for structs/enums.
- Keep modules focused by domain (`commands`, `api`, `config`, `output`) rather than mixing concerns.
- Prefer explicit, user-facing CLI argument names and help text consistent with existing clap patterns.

## Testing Guidelines
- Add tests in `tests/*_cli.rs` for user-visible CLI behavior, parsing, and serialization expectations.
- Name tests descriptively in `snake_case` (example: `parse_trending_with_episode_mode`).
- Cover both success and validation/error paths when adding flags, arguments, or endpoint behaviors.
- No fixed coverage percentage is configured; quality gate is passing `cargo test`, `cargo fmt`, and strict `clippy`.

## Commit & Pull Request Guidelines
- Follow the existing Conventional Commit style seen in history: `feat: ...`, `fix: ...`, `ci: ...`.
- Keep commit subjects imperative and scoped to one logical change.
- PRs should include: purpose, key changes, test evidence (commands run), and linked issue(s) if applicable.
- For CLI-facing changes, include example invocations/output snippets in the PR description.

## Security & Configuration Tips
- Never commit Podcast Index credentials.
- Configure secrets locally via `podcast-cli config set --api-key ... --api-secret ...`.
- Verify masked config output with `podcast-cli config show` before sharing logs.
