# Podcast CLI - 项目路线图

## 项目概览

**目标：** 构建一个 Rust CLI 工具，用于访问 Podcast Index API，专为 AI Agent 设计，支持 JSON/表格双输出模式。

**参考项目：** Polymarket CLI（agent-friendly 设计典范）

**技术栈：**
- Rust 2021 Edition
- clap（CLI 框架）
- reqwest（HTTP 客户端）
- serde/serde_json（序列化）
- prettytable-rs（表格输出）
- confy（配置管理）
- tokio（异步运行时）
- anyhow/thiserror（错误处理）

---

## 核心架构

```
podcast-cli/
├── src/
│   ├── main.rs           # 入口点
│   ├── cli.rs            # CLI 定义
│   ├── commands/         # 命令实现（按阶段扩展）
│   │   ├── mod.rs
│   │   ├── search.rs
│   │   ├── show.rs
│   │   ├── episodes.rs
│   │   ├── trending.rs
│   │   ├── recent.rs
│   │   ├── categories.rs
│   │   ├── stats.rs
│   │   └── config.rs
│   ├── api/              # API 客户端层
│   │   ├── mod.rs
│   │   ├── client.rs     # SHA1 认证
│   │   ├── endpoints/    # API 端点封装
│   │   └── types.rs      # API 类型定义
│   ├── config/           # 配置管理
│   │   ├── mod.rs
│   │   └── manager.rs
│   ├── output/           # 双输出格式化器
│   │   ├── mod.rs
│   │   ├── json.rs
│   │   └── table.rs
│   └── error.rs          # 错误类型
├── tests/                # 集成测试
├── Cargo.toml
└── README.md
```

**关键设计决策：**
- ✅ SHA1 认证（Podcast Index API 要求）
- ✅ 双输出模式（JSON for agents, Table for humans）
- ✅ 跨平台配置（由 `confy` 按操作系统写入对应配置目录）
- ✅ 模块化命令结构（易于扩展）
- ✅ 异步 HTTP 客户端（高性能）

---

## 三阶段实施计划

### Phase 1: MVP - 核心搜索功能 ✅

**目标：** 可用的基础 CLI，支持搜索和查看播客详情

**功能清单：**
- [x] 项目初始化（Cargo + 依赖）
- [x] API 客户端（SHA1 认证）
- [x] 配置管理（API key/secret）
- [x] 输出格式化器（JSON/Table）
- [x] `podcast search <term>` - 搜索播客
- [x] `podcast show <id>` - 查看播客详情
- [x] `podcast config` - 配置管理

**成功标准：**
- [x] 所有命令无错误执行
- [x] JSON 输出可被 jq 解析
- [x] 表格输出可读性良好
- [x] 配置持久化正常
- [x] API 认证成功

**详细计划：** [plans/phase-1-mvp.md](plans/phase-1-mvp.md)

**预计工作量：** 10 个任务，约 2-3 天

---

### Phase 2: Episodes & Trending ✅

**目标：** 扩展到单集和热门内容功能

**功能清单：**
- [x] `podcast episodes <feed-id>` - 列出播客的所有单集
- [x] `podcast episode <episode-id>` - 查看单集详情
- [x] `podcast trending` - 热门播客
- [x] `podcast trending --episodes` - 热门单集

**新增 API 端点：**
- `/episodes/byfeedid`
- `/episodes/byid`
- `/podcasts/trending`
- `/episodes/trending`

**成功标准：**
- [x] 单集列表支持分页
- [x] 热门内容实时更新
- [x] 输出格式与 Phase 1 一致
- [x] 覆盖核心 CLI 解析与输出测试

**详细计划：** [plans/phase-2-episodes.md](plans/phase-2-episodes.md)

**预计工作量：** 6-7 个任务，约 1-2 天

---

### Phase 3: Advanced Features ✅

**目标：** 完善高级功能和数据分析

**功能清单：**
- [x] `podcast recent` - 最新更新的单集
- [x] `podcast recent --feeds` - 最新添加的播客
- [x] `podcast categories` - 浏览分类
- [x] `podcast stats` - 统计信息

**新增 API 端点：**
- `/recent/episodes`
- `/recent/feeds`
- `/categories/list`
- `/stats/current`

**成功标准：**
- [x] 支持时间范围过滤（`--before` / `--since`）
- [x] 分类浏览完整（table/json）
- [x] 统计数据清晰展示（含千分位格式化）
- [x] 文档与示例同步更新

**详细计划：** [plans/phase-3-advanced.md](plans/phase-3-advanced.md)

**预计工作量：** 6-7 个任务，约 1-2 天

---

## 开发流程

### 每个阶段的标准流程：

1. **规划** - 阅读详细计划文档
2. **实现** - 按任务顺序执行
3. **测试** - 单元测试 + 集成测试
4. **验证** - 端到端测试
5. **文档** - 更新 README 和 API 文档
6. **Review** - 代码审查
7. **发布** - 版本标记

### 质量标准：

- ✅ 测试覆盖率 ≥ 80%
- ✅ `cargo clippy` 无警告
- ✅ `cargo fmt` 格式化
- ✅ 提交信息符合规范
- ✅ 文档完整

---

## 依赖关系

```
Phase 1 (MVP)
    ↓
Phase 2 (Episodes & Trending)
    ↓
Phase 3 (Advanced Features)
```

**关键依赖：**
- Phase 2 依赖 Phase 1 的 API 客户端和输出系统
- Phase 3 依赖 Phase 1/2 的所有基础设施

---

## 当前状态

**当前阶段：** Phase 3 - Advanced Features Completed

**下一步：** 发布前验收（`fmt`/`clippy`/`test`）并准备 release 包

**进度追踪：**
- Phase 1: 已完成
- Phase 2: 已完成
- Phase 3: 已完成

---

## 文档结构

```
podcast-cli/
├── ROADMAP.md              # 本文件 - 总体路线图
├── README.md               # 用户文档
├── plans/
│   ├── phase-1-mvp.md      # Phase 1 详细计划
│   ├── phase-2-episodes.md # Phase 2 详细计划
│   └── phase-3-advanced.md # Phase 3 详细计划
└── docs/
    ├── API.md              # API 参考文档
    └── EXAMPLES.md         # 使用示例
```

---

## Rust 生态系统优势

**为什么选择 Rust：**
- 🚀 **性能**：编译后的二进制文件启动快，内存占用低
- 🔒 **安全**：编译时保证内存安全和线程安全
- 📦 **分发**：单个二进制文件，无需运行时
- 🛠️ **工具链**：cargo 提供完整的构建、测试、发布工具
- 🌐 **跨平台**：轻松编译到 Linux/macOS/Windows

## 参考资源

- [Podcast Index API 文档](https://podcastindex-org.github.io/docs-api/)
- [Polymarket CLI](https://github.com/Polymarket/polymarket-cli) - 设计参考
- [clap 文档](https://docs.rs/clap/)
- [reqwest 文档](https://docs.rs/reqwest/)
- [Rust CLI Book](https://rust-cli.github.io/book/)
