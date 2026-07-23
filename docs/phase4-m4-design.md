# MVA Phase 4 M4 Architecture Design — Configuration Loading + Warning System

**Revision 3 (Final)**——已依据 M4 Architecture Review Final 逐条修正，可提交
**Prerequisite:** Phase 4 M1/M2/M3 已完成

---

## 0. Revision 3 最终修订摘要

| 审核条目 | 处理 | 位置 |
|---|---|---|
| Final 1: Known Limitations 增加 file-level fallback 粒度说明 | 新增 §10.5，明确 M4 为 file-level fallback；不实现 field-level recovery；属于未来阶段 | §10.5 |
| Final 2: 修正 Windows CWD 描述 | §4.2 重写：CWD depends on launcher/environment, is not guaranteed to match exe directory; exe-relative is preferred because deterministic | §4.2 |
| Final 3: 增加 missing app.toml in existing config dir 测试 | §9.1 新增测试 #8：config dir 存在但不含 `app.toml` → 默认值 + 空 warnings | §9.1 |

核心设计（loader 归属 / API / 双遍解析 / Vec\<String\> warnings / UI+stderr 双通道 / startup 不修改 / Engine 不修改 / 仅 app.toml / 不支持 runtime reload）保持不动。

---

## 1. Goal

将 MVA 从"代码默认值即配置"升级为"用户磁盘文件驱动配置"。具体实现三项能力：

1. **加载** `config/app.toml`——文件存在则读、解析、使用；不存在则静默内置默认值。
2. **Warning 可见**——未知字段、TOML 语法错误、字段值类型错误 → 生成 warning 进入 UI 面板 + stderr，绝不阻止启动。
3. **autoplay 终于受配置控制**——`autoplay_on_open = false` 真正阻止 Play，M3 的接线（`activate_project` 接受 `bool` 参数）在 M4 首次接入非默认值。

**零 Engine 改动、零新 crate、零 service layer。**

---

## 2. Current Architecture Analysis

### 2.1 配置数据流——M3 状态

```
config/app.toml (磁盘上存在，autoplay_on_open=true)
        │
        │    ✗ 从未被代码读取
        ▼
AppConfig::default()   ← main.rs:32 (硬编码)
        │
   autoplay = app_cfg.general.autoplay_on_open   ← main.rs:45 (始终 true)
        │
        ├─→ startup::boot(..., autoplay)         ← main.rs:48
        │         │
        │         ├─→ activate_project(..., autoplay)    ← CLI 路径
        │         └─→ build_on_effect(..., autoplay)     ← 闭包捕获，UI 路径
        │                    └─→ activate_project(..., autoplay)
        │
        └─→ eframe 窗口尺寸 = app_cfg.window.width/height
```

**插入磁盘文件只需要改一行**（`AppConfig::default()` → `loader()`），其余管线全部不动。这是 M3 设计刻意保留的接缝。

### 2.2 已有配置结构体与 from_toml

| 结构体 | 位置 | `from_toml` | 字段数 |
|---|---|---|---|
| `AppConfig` | `mva-core::config::app` | `Result<Self, CoreError>` | 2 sections / ~6 fields |
| `AnimationConfig` | `mva-core::config::animation` | `Result<Self, CoreError>`（含校验） | 1 section / ~5 fields |
| `RendererConfig` | `mva-renderer::config` | `Result<Self, String>` | 空结构体 |
| `AudioConfig` | `mva-core::config::audio` | 存在 | ~3 fields |
| `LyricsConfig` | `mva-core::config::lyrics` | 存在 | ~2 fields |

**M4 只加载 `app.toml`。** 其他文件遵循同一模式，在后续里程碑按需激活——不在本次范围。

### 2.3 关键调用链（M3，即将被 M4 替换）

- `main.rs:32` 仍用 `AppConfig::default()`—M4 替换为 config loader。
- `main.rs:45` extract `autoplay` bool—M4 不变（来源从 default 变为文件值）。
- `startup::boot(..., autoplay: bool)`（`startup.rs:54`）—M4 **不变**。
- `activate_project(..., autoplay: bool)`（`startup.rs:134`）—M4 **不变**。
- `build_on_effect(..., autoplay: bool)`（`startup.rs:198`）—M4 **不变**。
- `MvaUiApp::new`—M4 新增 `config_warnings` 参数（当前签名无此参数）。
- `settings::show()`—M4 新增 `config_warnings` 参数。

### 2.4 Loader 边界声明

`config::loader` 属于 **infrastructure** 层，不属于 runtime：

| 允许的调用方 | 禁止的依赖 |
|---|---|
| **composition root** (`main.rs`)——启动时唯一调用点 | **Engine 状态机**——engine.rs 绝不 import loader |
| **集成测试**（经 `load_app_config(Some(&temp_dir))` 注入） | **startup.rs**——启动逻辑不直接读文件 |
| **未来：Editor 二进制** | **mva-ui**——UI panels 只消费 `Vec<String>`，不触发加载 |

- loader 在 `main()` 开头执行一次，产出 `(AppConfig, Vec<String>)`——**一次调用，两个产物，全 session 有效。**
- loader 不提供 update / reload / watch 方法——见 §3.4 生命周期。
- loader 不依赖 engine / audio / ui 的任意类型——仅依赖 `std::env`, `std::fs`, `toml`, `serde`。

---

## 3. Config Loader Design

### 3.1 归属：`mva-core::config::loader`

| 方案 | 判定 |
|---|---|
| A: `mva-core::config::loader`（采纳） | 结构体 + `from_toml` + 文件发现 + unknown-key 检查共处同一模块；Editor 零复制；Engine 状态机本身保持零 I/O（loader 是 infrastructure，不是 runtime） |
| B: `mva-player` binary 内 | 文件 I/O 归属 application layer；但 Editor 到达时需全量复制 loader + whitelist + unknown-key 逻辑 |

采纳方案 A 的理由：

1. `from_toml` 方法已在此 crate——从"将字符串解析为 struct"到"从磁盘读取字符串再解析"是 Module 层级的自然延伸，不是架构层次的跃迁。
2. Editor 二进制将需要相同逻辑——方案 B 强制复制。
3. `mva-core` 的 purity 约束针对 **Engine 状态机**（不含文件 I/O），不是整个 crate。Config module 是基础设施，与 Engine 解耦——`engine.rs` 的构造接受 `AppConfig`，从哪里来它不关心。

### 3.2 API 形状

```rust
// mva-core::config::loader (NEW module)

/// Load AppConfig from the given config directory, collecting
/// warnings for unknown fields or parse failures.
///
/// `config_dir` is the directory that directly contains `app.toml`
/// (i.e., `<some_root>/config/`, not `<some_root>`).
///
/// - `Some(path)`: skip discovery; read `path/app.toml` directly
///   (for testing with a temp directory).
/// - `None`: auto-discover — try `<exe_dir>/config/` first,
///   then `<cwd>/config/`.  Fall back to built-in defaults if
///   neither exists or neither contains a readable `app.toml`.
///
/// Missing-file is a normal first-run state — no warning is
/// produced; built-in defaults are returned silently.
pub fn load_app_config(config_dir: Option<&Path>) -> (AppConfig, Vec<String>);
```

命名说明：`config_dir` 接受的是**直接包含 `app.toml` 的目录**——即 `config/` 本身，不是其父目录。测试中构建 tempdir，在其中写入 `app.toml`，传入 `Some(&tempdir)`——无需额外的 `config/` 子目录。

### 3.3 双遍解析——同时获得类型安全配置与 unknown-key warning

核心事实：serde 的默认行为是**忽略 unknown fields**（不是拒绝）。因此 unknown field 不会导致 Pass B 失败——用户的合法字段值得以保留，仅 unknown key 被 warning。

| Pass | 解析 | 产物 | 失败时 |
|---|---|---|---|
| **A: unknown-key 检查** | `toml::from_str::<Value>(&content)` → 遍历 section/key tree，与 known-keys 白名单对比 | `Vec<String>`——每条 unknown key 一个 warning | —（Value parse 极少失败：合法 TOML 但含 unknown key → 成功） |
| **B: 类型化解析** | `AppConfig::from_toml(&content)` (serde derive，ignore unknown by default) | `AppConfig`——所有已知字段来自文件值 | 语法损坏 / required 字段缺失或类型错误 → 整文件 fallback 到 default + 一条 warning |

**关键行为矩阵——Pass A + Pass B 的各种组合**：

| 场景 | `app.toml` 内容 | Pass A | Pass B | 最终 AppConfig | Warnings |
|---|---|---|---|---|---|
| 全合法 | 所有字段合法 | 空 | 成功 | 文件值 | 空 |
| unknown field + 所有 required 字段完整 | `[general]` 含 `unknown_opt = true` 且 `volume` / `language` 完整 | 1 warning: "unknown key `unknown_opt`" | **成功**（serde 忽略 unknown） | 文件值（合法字段保留） | 1 |
| unknown field + required 字段缺失 | `[general]` 含 `unknown = true`，但缺 `volume` | 1 warning: "unknown key `unknown`" | **失败**（`volume` missing, no serde default） | **全部默认值** | 2（Pass A 的 unknown warning + Pass B 的 "could not be parsed: missing field `volume`"） |
| unknown field + required 字段类型错误 | `[general] volume = "loud"` | 0 warning（`volume` 是 known key） | **失败**（type error） | **全部默认值** | 1（"could not be parsed: invalid type: string \"loud\", expected f32"） |
| 语法损坏 | `general]`（缺 `[`） | **失败**（toml::Value parse 失败） | 未到达 | **全部默认值** | 1（"could not be parsed: ..."） |
| 文件缺失 | 文件不存在 | 未执行 | 未执行 | **全部默认值** | **空**（首运行常态） |

**白名单维护**：expected-keys 表置于 `loader.rs` 顶部私有常量块，带 `// Keep in sync with AppConfig/GeneralConfig/WindowConfig in app.rs` 注释。三张表共约 10 行，位于与使用方同一文件内。当 struct 新增字段时，白名单在同行可见。

### 3.4 配置生命周期

```
┌─ 启动 ────────────────────────────────────────────────────────────┐
│ main.rs                                                           │
│   load_app_config(None) ─→ (AppConfig, Vec<String>)               │
│        │                     │                                    │
│        │  autoplay bool      │  config_warnings                   │
│        ▼                     ▼                                    │
│   startup::boot(autoplay)   MvaUiApp::new(warnings)               │
│        │                     │                                    │
│        ▼                     ▼                                    │
│   activate_project          settings panel 渲染（全 session 显示） │
│   (当前值不会变化)                                                 │
└────────────────────────────────────────────────────────────────────┘
```

**强制性约束**：

- 配置**仅在 `main()` 开头加载一次**。
- session 期间**不支持重新加载**（无 `reload()` 函数，无 SIGHUP 监听，无 `--reload-config` flag）。
- `autoplay_on_open` 的值在 session 内为常量——用户不能在运行时切换自动播放行为后不重启就生效。
- Warning 列表在 session 内为常量——用户修正了 `app.toml` 也不会在当前窗口消失。

这是 M4 的显式设计边界。**runtime config reload 属于后续阶段**（需解决：原子 reload、warning 清除、已持有引用的刷新策略——均不在 Phase 4）。

---

## 4. Configuration Discovery Strategy

### 4.1 Phase 4 查找顺序（`config_dir: None` 时）

```
1. <exe 所在目录>/config/app.toml     ← 安装 / 便携包
2. <当前工作目录>/config/app.toml     ← 开发 / cargo run
3. 内置默认值                          ← 两个目录都不存在
```

取第一个**存在且包含 `app.toml`** 的 `config/` 目录；该文件不可读 → 退至下一层。

### 4.2 为什么 exe 优先

CWD depends on the launcher and execution environment——it is not guaranteed to match the executable's directory. When launched via file-manager double-click or shell "Open With", the CWD is determined by the launching process, not by the executable's location. Therefore CWD-relative config resolution is non-deterministic for end-user launch scenarios.

Exe-relative config resolution is preferred because `std::env::current_exe()` returns a deterministic path: the actual location of the running binary. This makes `<exe_dir>/config/` the reliable path for installed or portable deployments, regardless of how the process was launched. CWD-relative resolution (`<cwd>/config/`) serves as a secondary path, providing zero-config convenience for `cargo run` and terminal-based development workflows.

### 4.3 执行细节

- `std::env::current_exe()` 在极少数平台可能失败（WASM, FreeBSD 特定配置）——失败时跳过步骤 1，退至步骤 2。不是 fatal。
- 没有任何路径规范化或符号链接展开——使用 `std::fs::metadata` 的默认行为。

---

## 5. Warning System Design

### 5.1 数据流

```
Config Loader (mva-core::config::loader)
        │
    (AppConfig, Vec<String>)        ← 一次调用产出两者
        │           │
        │           ├──→ stderr (eprintln! 每一条)    ← 开发者辅助
        │           │
        │           └──→ MvaUiApp::new(..., warnings) ← 用户主通道
        │                         │
        │                    settings panel renders
        │
        └──→ main.rs 使用 AppConfig
```

### 5.2 Warning 数据

`Vec<String>`——无包装结构体。每条独立文本，顺序无关。示例：

- `"unknown config key in [general]: unknown_option"`
- `"unknown config section: [network]"`
- `"config/app.toml could not be parsed: missing field 'volume' at line 6. Using defaults."`
- `"config/app.toml could not be read: Permission denied (os error 5). Using defaults."`

### 5.3 Warning 来源汇总

| 检测层 | 来源 | 每文件最多 warnings |
|---|---|---|
| Pass A (Value tree) | 未知顶层 section | N（每 section 一条） |
| Pass A (Value tree) | 已知 section 内的未知 key | N（每 key 一条） |
| Pass A (Value tree) | TOML 语法损坏（`toml::from_str::<Value>` 失败） | 1——直接跳过 Pass A，仅 Pass B 报 parse 错误 |
| Pass B (serde) | Parse 失败（required 字段缺失 / 类型错误） | 1（整文件一条，含 serde 错误原文） |
| 文件 I/O | `std::fs::read_to_string` 失败（权限等） | 1 |

### 5.4 UI 渲染方式

在 `settings::show()` 底部新增条件块：

- `config_warnings` 非空时 → amber 标题 "Configuration Warnings (using defaults where needed):" + 逐条 label。
- 为空时 → 不渲染任何额外 UI。
- 不自动消失、不折叠——session 期间始终可见（鼓励用户修正配置）。

### 5.5 stderr 二次输出

每条 warning 同时在 `eprintln!` 输出一行 `[mva-player] config: {warning}`。理由：

- `cargo run` 开发者的终端中即时可见——不需要看 settings panel；
- 用户双击启动无控制台时，stderr 不可见——此时 UI panel 是主通道；
- 两个通道互补，不互斥，增加一行 `eprintln!` 不引入新通道设计。

---

## 6. Error Handling——分类与启动决策

| 场景 | 分类 | 行为 |
|---|---|---|
| `config/app.toml` 在 exe/CWD 均不存在 | 正常（首运行） | 默认值，**无** warning |
| TOML 语法损坏 / required 字段缺失 / 类型错误 | Warning | 默认值 + 1 warning |
| 文件存在但无读权限 | Warning | 默认值 + 1 warning |
| 已知 section 内有未知 key | Warning(s) | **合法字段保留文件值**（serde 忽略 unknown），每条 unknown key 1 warning |
| 未知顶层 section | Warning(s) | 同合法字段保留；每 unknown section 1 warning |
| 所有字段合法 | — | 文件值生效，0 warning |

**Config 永远不会导致启动终止。** 它的语义是 "best-effort"——有文件则读，读不了/读不懂则退至默认值。唯一的 Fatal 路径仍是音频设备不可用（M2），M4 不改动。

---

## 7. autoplay Integration

### 7.1 M3 → M4 的唯一变化

```
// main.rs (M3)                         // main.rs (M4)
let app_cfg = AppConfig::default();     let (app_cfg, warnings) = load_app_config(None);
let autoplay = app_cfg.general...       let autoplay = app_cfg.general.autoplay_on_open;
                                        //                                  ↑ 可能为 false
```

**startup::boot / activate_project / build_on_effect 完全不变**——M3 的 `autoplay: bool` 参数已就位，M4 只改变传入的值的来源。

### 7.2 autoplay=false 完整数据流

```
config/app.toml: autoplay_on_open = false
        │
load_app_config(None) → AppConfig { general: { autoplay_on_open: false } }
        │
main.rs: autoplay = false
        │
startup::boot(..., false)
        ├─→ activate_project(..., false) → LoadProject → Ready (无 Play effect)
        └─→ build_on_effect captures false → UI OpenFile → LoadProject → Ready
```

### 7.3 验证矩阵

| 测试层 | autoplay=true | autoplay=false |
|---|---|---|
| **Config serde** | TOML `autoplay_on_open = true` → 字段 `true` | TOML `autoplay_on_open = false` → 字段 `false` |
| **loader 产出** | `load_app_config(Some(&temp_dir))` 返回 `AppConfig` 含 autoplay=true | 同左，含 `false` |
| **序列（activate_flow）** | LoadProject + Play → effects 含 `Audio(Play)`（M3 已有） | LoadProject 无 Play → effects 空，state=Ready（M3 已有） |
| **端到端（M6 人工）** | `cargo run -- ...` 自动播放 | 修改磁盘 `app.toml` 为 false → 不播放；改回 true → 恢复 |
| **`--demo`** | 结构性免检——demo 路径不询问 autoplay 变量 | 同左 |

---

## 8. File Changes

| 文件 | 变更 | 估计 |
|---|---|---|
| **`crates/mva-core/src/config/mod.rs`** | + `pub mod loader;` | +1 |
| **`crates/mva-core/src/config/loader.rs`** | **NEW**——`load_app_config()`; `find_config_dir()`（exe→CWD→None）; Pass A—`check_unknown_keys()`（Value tree + known-keys 白名单）; Pass B—`AppConfig::from_toml` | +85 |
| `crates/mva-core/src/config/app.rs` | **不变**（from_toml / Default / autoplay_on_open 字段均已就位） | 0 |
| **`crates/mva-player/src/main.rs`** | 第 32 行 `AppConfig::default()` → `load_app_config(None)` 解构为 `(app_cfg, warnings)`; warnings 传入 `MvaUiApp::new`。其余行不变 | +3, -1 |
| `crates/mva-player/src/startup.rs` | **不变**（boot / activate_project / build_on_effect 均不接触） | 0 |
| **`crates/mva-ui/src/app.rs`** | `MvaUiApp` + `config_warnings: Vec<String>`; `new()` + 参数; `ui()` 中透传 | +5 |
| **`crates/mva-ui/src/panels/settings.rs`** | `show()` 签名 + `config_warnings: &[String]`; 底部渲染 warning list | +10 |
| **`crates/mva-core/tests/config_tests.rs`** | 扩展 loader 测试（tempdir 中 9 场景——见 §9） | +75 |
| `config/app.toml` | **不变** | 0 |

### 显式禁止修改

| 禁改项 | 原因 |
|---|---|
| `mva-core/src/engine.rs` / `state.rs` / `command.rs` / `effect.rs` | 规则——M4 不触碰 Engine |
| `mva-core/src/config/app.rs` | M3 已就位 |
| `mva-player/src/startup.rs` | activate_project API 稳定 |
| `mva-format/**` / `mva-audio/**` / `mva-timeline/**` / `mva-scene/**` / `mva-renderer/**` | 正交 |
| `crates/mva-player/src/cli.rs` | CLI surface 不增不减 |

---

## 9. Test Plan

### 9.1 Config loader 单元测试（mva-core）

均经 `load_app_config(Some(&temp_dir))` 注入，**不依赖磁盘真实文件**。

| # | 测试 | 设置 | 预期 AppConfig | 预期 Warnings |
|---|---|---|---|---|
| 1 | 全合法 | `app.toml` 所有字段含 `autoplay_on_open = true` | 匹配文件值 | **空** |
| 2 | autoplay=false | `autoplay_on_open = false`，其余完整 | `autoplay_on_open == false` | **空** |
| 3 | unknown field + required 字段完整 | `[general]` 含 `unknown_opt = true` 且 `volume = 0.8` 完整 | **文件值**（`volume = 0.8` 保留，serde 忽略 unknown） | 1: `"unknown config key in [general]: unknown_opt"` |
| 4 | unknown field + required 字段缺失 | `[general]` 含 `unknown = true`，但缺 `volume` | **全部默认值**（Pass B parse 失败 ← missing required field） | 2: 未知 key warning + "could not be parsed: missing field `volume`" |
| 5 | TOML 语法错误 | `general]`（缺 `[`） | **全部默认值** | 1: "could not be parsed: expected a table key..." |
| 6 | 文件缺失——config dir 不存在 | `config_dir` 指向不存在目录 | **全部默认值** | **空** |
| 7 | 未知 section | `[network] timeout = 5` | **全部默认值** | 1: "unknown config section: [network]" |
| 8 | 文件缺失——config dir 存在但无 app.toml | temp_dir 存在（含其他文件或不含），但无 `app.toml` | **全部默认值** | **空** |
| 9 | 发现顺序 | exe 目录 + CWD 各有不同值 | 使用 exe 目录的文件值 | 空 |

测试 3 是本设计最关键的测试——验证 **"unknown field = warning but legal fields preserved"** 的行为，而非整文件回退。

测试 8 覆盖 config 目录存在但目标文件缺失的边界：这是正常的首运行状态（`config/` 目录可能因其他配置文件而存在，但 `app.toml` 尚未创建），应静默回退默认值，不产生任何 warning。

### 9.2 autoplay 序列集成（已有，不新增）

`activate_flow.rs`（M3）已覆盖 autoplay=true/false 的命令序列。M4 不需要新增——autoplay 值的来源变了（default → file），但下游行为已被 M3 锁定。

### 9.3 Warning 渲染（人工，M6）

settings panel UI 属于纯视图——人工 checklist 覆盖。

### 9.4 存量回归

每提交：`cargo test --workspace` + `cargo clippy --workspace` + `cargo fmt --check`。

---

## 10. Known Limitations

### 10.1 仅加载 app.toml

`animation.toml` / `renderer.toml` 等仍保持默认值。`loader.rs` 建立的模式（`load_*_config()` + 双遍解析 + whitelist）可复用于其他文件——后续里程碑按需激活。

### 10.2 配置不支持运行时重载（§3.4）

M4 配置为**启动时一次性加载**。`autoplay_on_open` 和警告列表在 session 期间为常量。

### 10.3 白名单与 struct 定义需手动同步

当 `AppConfig` / `GeneralConfig` 新增字段时，`loader.rs` 中的 known-keys 白名单必须同步更新——白名单与 struct 定义位于同一 crate 同一目录，注释已标记同步要求。

### 10.4 `current_exe()` 在极少数平台可能不可用

退至 CWD 查找 → 最终退至默认值。不会 crash。

### 10.5 File-level fallback 粒度

M4 当前采用 **file-level fallback**：如果 `app.toml` 内任一 required 字段缺失、任一字段值类型错误或 TOML 语法整体损坏，**整个 `AppConfig` 回退到 `Default`**——即 `window` 和 `general` 两个 section 的全部字段同时丢失文件值。见 §3.3 行为矩阵的场景 4 和 5。

M4 **不实现 field-level recovery**。不逐字段判断 "window.width 合法但 general.volume 非法 → 保留 window.width + 回退 general.volume"。

**Field-level validation and partial recovery 属于未来阶段**，需要：per-field serde default 普及、字段级错误收集与合并、以及明确的字段优先级语义（file value vs default vs error-fallback）。当前 M4 的整文件粒度已在 `Volume = "loud"` 导致窗口尺寸也一并回退的场景中体现了此限制——用户的纠正路径是修复那个字段后重启。

---

## 11. Decisions

1. **Loader 归属**：`mva-core::config::loader`（采纳，infrastructure layer，Engine 不依赖）。
2. **API 语义**：`load_app_config(config_dir: Option<&Path>)`——`config_dir` 为直接包含 `app.toml` 的目录（采纳，测试简易）。
3. **Unknown-key 检测**：双遍解析 `toml::Value` + `serde` derive（采纳，warn-and-continue，合法字段保留）。
4. **M4 范围**：仅 `app.toml`（采纳，autoplay 是唯一用户可见变更）。
5. **Warning 通道**：UI panel + stderr（采纳，双通道互补）。
6. **运行时 reload**：明确不支持（采纳，M4 设计边界）。
7. **白名单位置**：`loader.rs` 内私有常量，与使用逻辑同文件（采纳，修改时单文件可见，不污染纯数据 struct）。
8. **Fallback 粒度**：file-level（采纳，field-level recovery 属于未来阶段——见 §10.5）。

---

**状态：FINAL——可提交实现。**
