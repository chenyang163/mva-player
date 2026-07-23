# MVA Phase 4 Architecture Design — Application Workflow & Project Loading

**Revision v2**
**Status:** APPROVED — proceeding to M1
**Date:** 2026-07-23
**Based on:** Phase 4 Design v1 + 审核报告（结论：Approve with Changes）；所有事实性修正已逐条对照源码复核

---

## 0. Revision v2 修订摘要

v1 的核心方向**全部保留**：composition-root 重构、三个启动模式、CLI 表面、无参启动 = idle 窗口、clap 依赖决策、里程碑划分。v2 仅修复审核指出的问题：

| 审核条目 | 处理 | 位置 |
|---|---|---|
| **Must Fix 1:** 无音频设备承诺不可交付（UI 构造必须有 clock） | 撤销"无设备浏览模式"；改为优雅终止通道：最小错误窗口 + stderr + exit 1，不新增 fallback clock | §3.5 |
| **Must Fix 2:** `ProjectLoadError` 无 `Display`, 错误路径按字面无法编译 | `impl Display` 从 optional 提升为 **M1 必需前置** | §7、§12 |
| **Must Fix 3:** 配置路径解析策略缺失;fallback 粒度未定义 | 新增完整配置系统设计：查找顺序、逐文件 fallback、warning 收集与 UI 可见通道 | §6 |
| **Must Fix 4:** test_project.rs 只删不建, 违背 phase2:613 计划 | 执行完整计划：删除 + 建立 `tests/fixtures/` (minimal.mva/wav/lrc); startup_flow 以 fixtures 为主 | §10、§11 |
| **Must Fix 5:** 运行期打开失败的现场语义零记录 | 记录为 Known Limitation; 补 engine 级行为锁定测试; UI 增加恢复提示 | §9.1、§11 |
| **Should Fix 6:** CLI 演进方向 (flags vs subcommands) 未声明;退出码无策略 | 新增 CLI 语法演进策略 + 退出码策略 | §4.3、§4.4 |
| **Should Fix 7:** activate_project 缺锁契约、I/O 线程位置、内部分界 | 重定义为 prepare/activate 两阶段 + 显式锁契约; 风险表补登运行期 UI 冻结 | §3.3、§9.2、§13 |
| **Should Fix 8:** 配置错误可观测性近零 (stderr 在 Windows GUI 场景不可见; unknown key 静默) | warning 进入 UI 可见通道 (settings 面板状态行); app 配置族采用 `deny_unknown_fields` | §6.3、§6.5 |
| **Should Fix 9:** `--demo` 是否受 autoplay 门控未规定; 字段归属倾向未记录 | 显式规定 demo 永远自动播放; 记录未来双前端时可能拆 `[player]` 表 | §3.4 |
| **Should Fix 10:** 配置 I/O 归属 (player binary vs mva-core) | 建议归属 `mva-core::config` (Editor 复用; Engine 仍零 I/O); 列为待确认决策 | §6.4、§14 |
| **事实修正:** `#[serde(default = true)]` 不是合法 serde 属性 | 改为"返回 true 的辅助函数"表述 | §3.4 |
| **补充:** Engine 不记录来源路径，未来 title/last_directory 需要 | 明确路径归属 = composition root; Phase 4 仅记录不消费 | §9.5 |

---

## 1. Phase 4 定位与范围

**Phase 4 = Application Workflow + Project Loading。** 它只负责让用户能够：

1. `mva-player` — 无参数启动，打开 idle 播放器窗口；
2. `mva-player --demo` — 启动内置 Phase 3 showcase（合成音频，永远自动播放）；
3. `mva-player <PATH>` — 打开指定的 MVA 项目（`.mva` manifest / 音频文件 / 松散目录）。

**Phase 4 不是：**

| 非目标 | 说明 |
|---|---|
| MVA format redesign | manifest schema、loader 逻辑零改动 |
| Editor architecture | Editor 是独立 binary、独立 composition root，本阶段只保证不被阻碍 |
| Async loading 实现 | 只做两段式边界设计 (§3.3)，不实现 worker thread |
| Asset system | `AssetRef`、mva-assets 均不动 |
| Plugin system | 与 composition root 正交，不触碰 |
| ZIP container | 维持 roadmap 4.1 planned 状态 |
| 无音频设备浏览模式 | 明确不提供 (§3.5) |

### 分类

| 分类 | 项 |
|---|---|
| **MUST** | CLI 解析 (positional path / `--demo` / `--help` / `--version`); 三个启动模式; **`impl Display for ProjectLoadError`**; 启动路径零 panic (含音频设备失败); 配置文件加载策略 (查找顺序 + 逐文件 fallback + UI 可见 warning); `tests/fixtures/`; activate_project 两阶段边界与锁契约文档化; 失败语义记录为 Known Limitation |
| **WILL NOT** | 新 crate; 新 engine 命令/状态; Engine 事务机制; async 加载实现; 格式改动; ZIP; fallback clock; 无设备浏览模式; 原生文件对话框/拖拽/recent files/headless |
| **DEFER** | 异步加载 (worker + completion dispatch); Engine transaction/revert; 配置版本迁移机制; 用户目录配置 (AppData); 窗口标题=项目标题; `last_directory` 写回; `--config`/`--fullscreen`/`--validate`/`--export` |

---

## 2. 现状事实基础（经审核复核）

- 加载管线已存在并在运行时被使用 (`main.rs:71-98` 的 `on_effect` 闭包); Phase 4 是 **composition-root 重构**，不是新机制建设。
- `PlaybackError::Unknown(String)` 已存在 (`state.rs:60`), Engine 无需改动即可承载错误详情 — 但 `ProjectLoadError` **只有 Debug 没有 Display** (`loader.rs:12`)，错误详情文本化是必修项。
- `test_project.rs` 是死文件 (`main.rs:7` 仅声明 `mod demo;`); `integration_flow.rs:20-22` 自述 builder 为内联复制。phase2-architecture.md:613 的计划原文是 "Remove `test_project.rs` from binary. **Replace with fixture files in `tests/fixtures/`**" — v2 执行完整计划。
- `from_toml` 系列已存在且有测试，但从未被 binary 调用；配置文件路径解析策略完全缺失。
- `MvaUiApp::new` 签名要求 `clock: Box<dyn PlaybackClock>` (`app.rs:25-31`)，每帧无条件读时钟 (`app.rs:46`) — 这是设备失败处理的设计约束。

---

## 3. 应用入口设计

### 3.1 Startup modes

```
enum StartupMode {
    Empty,
    Demo,
    OpenProject(PathBuf),
}
```

无参 → `Empty`; `--demo` → `Demo`; positional path → `OpenProject`; 两者同现 → clap conflict 错误。

### 3.2 Composition root 结构

```
crates/mva-player/src/
├── main.rs      ~60 行: parse → configs → subsystems → startup mode → UI
├── cli.rs       NEW: pure 解析 + StartupMode + 单元测试
├── startup.rs   NEW: 两个明确分隔的区域 (见下)
└── demo.rs      代码不变; doc header 改为"内置 --demo showcase"
```

**startup.rs 内部双区域：** 文件物理上包含两类寿命不同的代码，设计显式分区、但不拆第三个文件：

- **Bootstrap 区 (一次性)**: 三个启动序列、配置文件加载调用 — 只在 `main()` 开头运行一次；
- **Runtime service 区 (长期)**: `activate_project` — 整个会话期间被 UI 反复调用。

文档与模块头注释必须写明：`activate_project` 是运行时服务，因规模暂置于 startup.rs；未来规模增长时它是第一个被移出的候选。

### 3.3 activate_project: 两阶段边界

保留该抽象，内部逻辑边界重新定义：

| 阶段 | 内容 | 性质 | 当前执行位置 | 未来 async 化 |
|---|---|---|---|---|
| **prepare** | `loader.load(path)` → 解析 manifest/LRC/JSON → 解析音频来源路径 → 产出 `PreparedProject` (Project + 已解析音频路径) | 纯 I/O + parse, 不触碰 engine/audio | 调用线程同步执行 | 可整体移至 worker thread，产出经 completion 通道送回 |
| **activate** | `audio.load_file()` (音频切换) → `Engine::LoadProject` → 可选 `Play` → dispatch 音频 effects | 纯状态操作 | 主线程 (启动路径) / UI 线程 (运行期 effect dispatch, 锁外) | 始终留在主线程，原样复用 |

设计约束：

- **锁契约 (必须写入代码 doc 与本文档)**: 调用方在进入本 helper 时不得持有 Engine 锁; activate 阶段在内部自行加锁。当前两个调用点均满足 (UI 侧 `app.rs:83-88` 先释放锁再 dispatch effect)。
- **失败归属**: prepare 失败 → `PlaybackError::Unknown(Display 文本)`; activate 的音频切换失败 → `PlaybackError::DecodeFailed`。两者都经 `set_error` 进入 Error 状态, GUI 不退出。
- 两阶段在当前实现中是同一函数内的两个连续段落；分界是逻辑契约，保证未来 async 化时只有 prepare 需要挪动。

### 3.4 自动播放策略

MVA Player 的定位是 **MVA 格式播放器 + 可视化运行时展示器**，不是 Editor。`mva-player demo.mva` 是格式的最小演示闭环，open = play 与产品定位一致。

- 保留 `autoplay_on_open = true` (config/app.toml `[general]`)。
- **`--demo` 永远自动播放，不受该配置影响** — demo 是 showcase，这是显式规定，不是默认行为。
- 配置门控只覆盖两个真实加载入口 (CLI 启动路径 + UI OpenFile 路径)。
- **归属记录(不实施)**: `autoplay_on_open` 是播放器关注点，暂置于共享 `GeneralConfig`；未来 Editor 复用配置结构时拥有自己的配置，不共享该字段 — 届时可能需要拆分 `[player]` 表。现在建表是过度设计，仅记录倾向。
- **事实修正**: serde 的 `default` 接受函数路径而非字面量，实现时采用"返回 true 的辅助函数"形式; `AppConfig::default()` 同步包含 `true`。

### 3.5 音频设备失败处理

**撤销 v1 的"无设备也可浏览"承诺。** 该承诺要求 fallback clock，与"几乎无新机制"矛盾；Phase 4 明确不提供无音频设备浏览模式。

新设计 — 优雅终止通道：

1. `AudioPlayer::new()` 失败 → 不 panic；
2. 产生明确错误：完整 `AudioError` 文本写 stderr；
3. GUI 优雅失败：启动一个最小错误窗口（复用已有 eframe 依赖，不构造 `MvaUiApp`，仅展示错误文本 + 退出按钮，位于 startup.rs bootstrap 区）— 这是 Windows 双击无控制台场景下唯一的用户可见通道；
4. 用户关闭错误窗口 → 以 exit code 1 退出 (退出码策略见 §4.3)；
5. 启动路径上其余 `expect()` (音频源装载等) 同样改道此通道；
6. 兜底：若连窗口系统也不可用 (eframe 自身失败), stderr + exit 1 仍是最终行为。

不新增任何组件：无 fallback clock、无第二 UI 模式、无新依赖。

---

## 4. CLI 设计

### 4.1 语法表面

```text
Usage: mva-player [OPTIONS] [PATH]

Arguments:
  [PATH]  Project to open: .mva manifest, audio file (.mp3/.flac/.wav),
          or a loose project folder

Options:
      --demo       Play the built-in showcase project (synthetic audio)
  -h, --help       Print help
  -V, --version    Print version
```

`[PATH]` 使用 PathBuf value parser (OsStr, Windows 非 UTF-8 安全，与 Phase 2 的 `EngineEffect::LoadProject` 决策一致); `--demo` 与 `[PATH]` 互斥；不存在/不支持的 PATH 不是 CLI 错误，走 engine Error 状态 (§9.1)。

### 4.2 解析器依赖

clap 4 (derive), MIT OR Apache-2.0, 入 `docs/dependencies.md`; pico-args 记录为备选; hand-rolled 依 Rule 1 拒绝。

### 4.3 退出码策略

| Code | 含义 | 当前使用者 | 未来预留 |
|---|---|---|---|
| **0** | 正常退出 | 窗口正常关闭 | headless/validate 成功 |
| **1** | 启动/运行期致命错误 | 音频设备不可用 (§3.5) | headless 加载失败、validate 失败 |
| **2** | CLI 用法错误 | clap 解析失败 (默认行为) | 同左 |

仅一页策略记录，防止未来 headless 路径与脚本兼容性冲突；不实现任何新退出路径。

### 4.4 CLI 语法演进策略

- 未来复杂操作 (`--validate`/`--export`/`--headless`) 是命令模式而非 flag，将以 **subcommand** 形式引入: `mva-player validate x.mva`、`mva-player export x.mva`。
- 当前扁平语法被显式声明为未来语法的兼容子集: `mva-player x.mva` ≡ 未来默认 `play` 行为; `--demo` ≡ 未来 `demo` 子命令或保留 flag。
- Phase 4 不实现 subcommand；本节仅为防止 flag conflict 矩阵膨胀锁死错误语法方向。

---

## 5. 无参数默认行为

Empty 模式: idle 播放器窗口。engine 保持 `Stopped`、无 project; UI 渲染既有空 snapshot; settings 面板提示语保持。不自动播放 demo (惊吓用户), 不把 examples 资产变成运行时依赖。`--demo` 保留一键访问旧行为。

---

## 6. 配置系统设计

### 6.1 查找顺序 (Phase 4 定义，目录级解析)

```
1. <exe 所在目录>/config/     ← 双击启动、便携包场景
2. <当前工作目录>/config/     ← 开发场景 (cargo run)
3. 内置默认值                  ← 两者均不存在
```

取第一个存在的 config 目录，此后该目录内逐文件加载；两个目录都不存在 → 全部默认 + 一条 warning。不引入配置框架，仅一段解析策略。

### 6.2 fallback 粒度与 warning 收集

- **逐文件 fallback**: app.toml 损坏不连累 audio/animation/renderer.toml；
- **承认的连坐行为 (Known Limitation §9.3)**: 单文件内任一字段非法 (如 `volume = "loud"`) → 该文件整体回退默认。Phase 4 不做字段级隔离；
- 每次 fallback 产生一条结构化 warning (文件路径 + 原因)，汇集为 `Vec<String>` 交给 UI。

### 6.3 warning 可见性

stderr 在 Windows 双击/"打开方式"启动时不可见，不能作为唯一通道：

- **主通道**: settings 面板新增一行配置状态 (来源: 默认/目录路径/fallback + 原因)。`MvaUiApp::new` 增加一个 `config_warnings: Vec<String>` 参数 (mva-ui 签名级小改)；
- 辅通道: stderr 原文保留 (开发场景)。

### 6.4 归属决策

配置目录解析 + 逐文件 load-or-default + warning 收集，建议归属 `mva-core::config` (新增加载函数，非新 crate)：

- mva-core 已拥有配置结构与 `from_toml`，这是自然延伸；
- Editor 到达时直接复用，避免复制；
- 边界声明：这为 mva-core 引入只读文件 I/O，严格限定于配置加载; Engine 状态机本身保持零 I/O, purity 不受影响。
- 备选(不推荐)：留在 startup.rs, Editor 到达时复制。列为 §14 待确认项。

### 6.5 deny_unknown_fields

app 配置族 (`AppConfig`/`WindowConfig`/`GeneralConfig`) 增加 `#[serde(deny_unknown_fields)]`:`volme = 0.5` 这类拼写错误将从"静默忽略"变为"该文件 fallback + warning 指出未知字段"，与 §6.3 的 UI 通道形成完整可见性。权衡：降级运行 (旧二进制读新配置) 会 fallback 并提示 — pre-1.0 可接受，已记录。其余配置族 (animation/renderer/lyrics/audio) 同模式评估，无破坏则同批应用。

### 6.6 版本迁移策略

`AppConfig` 无 `config_version`; pre-1.0 声明: 配置结构允许破坏性变更，fallback 机制兜底。Phase 4 不实现迁移机制。

### 6.7 延期项

用户目录配置 (Windows AppData / XDG 等) 属于后续阶段；届时查找顺序扩展为四级，本节策略不变。

---

## 7. Crate 关系 (更新)

| 角色 | Phase 4 职责 | 变更 |
|---|---|---|
| **main.rs** (composition root) | 决定启动时装载什么；连接子系统；拥有 effect-dispatch 闭包；记录当前项目来源路径 (仅 `Option<PathBuf>`, 不消费，见 §9.5) | 重写为薄编排层；窗口标题 → `"MVA Player"` |
| **mva-core** (契约 + engine + config) | 拥有 Engine/命令/效果/配置结构 | 三处小改 (全部必修): ① `impl Display for ProjectLoadError`(手写 impl, 无新依赖，保留既有 derive); ② `GeneralConfig.autoplay_on_open`(辅助函数 serde default); ③ `mva-core::config` 增加目录解析 + 逐文件加载 + warning 收集 (§6.4); ④ app 配置族 `deny_unknown_fields`。Engine 状态机零改动 |
| **mva-format** (format engine) | `MvaLoader` 被 composition root 在两个入口调用 | 零变更 |
| **Runtime** (engine + effect loop + UI poll loop) | 稳态机制 | 零变更; Phase 4 只改喂给它的 bootstrap |

依赖规则：全部新连线止于 binary 与 mva-core; `mva-format → mva-player` 禁令维持；工作区唯一新增外部依赖是 binary 私有的 clap。

---

## 8. 数据流

### 8.1 启动流程 (含设备失败分支)

```
                        argv
                          │
                    cli::Cli::parse()
                          │
                   StartupMode
          ┌───────────────┼──────────────────┐
          ▼               ▼                  ▼
       Empty           Demo            OpenProject(path)
          │               │                  │
          │               │            prepare(path)
          │               │             ok ──┴── err
          │               │             │       └→ set_error(Display 文本)
          │               │             ▼          UI 打开并显示 Error
          │               │          activate(prepared)
          │               │          (audio 切换→LoadProject→Play?)
          ▼               ▼             ▼
   ┌────────────────────────────────────────────┐
   │ AudioPlayer::new() ──失败──→ 最小错误窗口    │
   │                            + stderr + exit 1│
   │ 成功 → eframe::run_native("MVA Player", ...) │
   └────────────────────────────────────────────┘
```

### 8.2 统一加载管线 (两入口汇聚, 标注两阶段)

```
 入口A (启动):   cli path ──────────────┐
                                        ▼
 入口B (运行期): UI OpenFile → EngineEffect::LoadProject
                                        │
        ┌── prepare: loader.load + 解析 (可同步; 未来 worker)
        │
        └── activate: audio 切换→LoadProject→Play? (主线程; 锁契约)
                                        │
                                  Engine 状态机
                                        │
 每帧 (不变): clock→update_position→snapshot→Renderer→DrawList→panels
```

---

## 9. 已知限制 (Known Limitations)

### 9.1 打开失败的现场语义

完整链路 (已核实): UI 打开 B → Engine 置 `Loading`、不发音频效果 (`engine.rs:136-140`) → 旧项目 A 若在播, 声音继续 → prepare 失败 → `set_error` → Error 状态, A 仍留在 engine → Play 被 `invalid_state` 拒绝 (`engine.rs:98-100`) → 唯一恢复路径是按 Stop (`engine.rs:111-113` 清 error)。

Phase 4 不修改 Engine。处置: 本节记录为 Known Limitation; settings 面板 Error 分支增加恢复提示 ("Press Stop to reset"); 补 engine 级测试锁定该行为 (§11)。未来由 Engine transaction/revert 机制解决 (Later)。

### 9.2 运行期打开的 UI 线程冻结

UI 打开第二个文件时，prepare 阶段 (文件 I/O + 解码探测) 在 egui 帧循环的 effect dispatch 中同步执行 (`app.rs:86-88` 锁外但仍在 UI 线程)。大型/慢速介质文件 → 窗口数秒无响应 (Windows 标"未响应")，期间 Engine 处于 `Loading`, 操作被拒。Phase 4 不实现 async; 风险表登记为 Medium; §3.3 两段式分界保证未来只需: worker thread + completion dispatch (新管道，含 `request_repaint`) + activate 原样复用。

### 9.3 单字段配置错误的整文件回退

见 §6.2: 字段非法 → 该文件整体回退默认 (warning 可见)。字段级隔离属于后续。

### 9.4 松散目录项目 duration = 0

roadmap 4.2 (duration probing) 维持 planned，不在本阶段。

### 9.5 项目来源路径的归属

Engine 只存 `Project` 不存路径。设计裁定：来源路径由 composition root 记录 (`Option<PathBuf>`); Phase 4 不消费它 — `last_directory` 写回、窗口标题 = 项目标题均为 DEFER, 届时无需改 Engine。

---

## 10. 文件变更清单

不新增 crate、服务层、管理器。仅: 两个 binary 模块 + mva-core 既有模块的小扩展 + 测试 fixtures。

| 文件 | 变更 |
|---|---|
| `crates/mva-player/Cargo.toml` | + clap (derive) |
| `crates/mva-player/src/cli.rs` | **NEW** — Cli / StartupMode / conflict 规则 / 单元测试 |
| `crates/mva-player/src/startup.rs` | **NEW** — bootstrap 区 (三启动序列、致命错误窗口、config 加载调用) + runtime service 区 (`activate_project`, prepare/activate 两段, 锁契约写入 doc) |
| `crates/mva-player/src/main.rs` | 重写薄编排 (~60 行); 窗口标题 `"MVA Player"`; 记录 `Option<PathBuf>` |
| `crates/mva-player/src/demo.rs` | 代码不变; header 改为"`--demo` 内置 showcase" |
| `crates/mva-player/src/test_project.rs` | 删除 (死文件; phase2:613 前半) |
| `crates/mva-player/tests/fixtures/` | **NEW** (phase2:613 后半): `minimal.mva` + `minimal.wav` (约 1 秒小 PCM, 数 KB, 生成后提交) + `minimal.lrc` (2 行)。测试经 `CARGO_MANIFEST_DIR` 相对定位, crate 内自洽 |
| `crates/mva-player/tests/startup_flow.rs` | **NEW** — 以 fixtures 为主 (见 §11) |
| `crates/mva-player/tests/demo_showcase.rs` | 代码不变; 角色重新定位为唯一真实资产冒烟测试 (assets 已在 docs/demo-assets.md 登记) |
| `crates/mva-core/src/loader.rs` | + `impl Display for ProjectLoadError` (必修) |
| `crates/mva-core/src/config/app.rs` | + `autoplay_on_open` (辅助函数 default); app 配置族 `deny_unknown_fields` |
| `crates/mva-core/src/config/` (mod 或新增 load 子模块) | + 目录解析 (§6.1) + 逐文件 load-or-default + warning 收集 (§6.4) |
| `crates/mva-ui/src/app.rs` | `MvaUiApp::new` + `config_warnings: Vec<String>` 参数 |
| `crates/mva-ui/src/panels/settings.rs` | 配置状态行; Error 分支恢复提示; help 文本刷新 |
| `config/app.toml` | + `autoplay_on_open = true` |
| **文档** | `docs/phase4-architecture.md` (本文件); `dependencies.md` (clap); `architecture.md` §13 (test_project 移除 + demo.rs 新角色); `phase2-architecture.md`:613 (标记 fixture 计划已执行); `README.md` (用法); `roadmap.md` / `project-status.md` / `CHANGELOG.md`; `examples/lyric_demo/README.md` (规范运行命令) |

**显式不动**: mva-format / mva-timeline / mva-scene / mva-renderer / mva-audio / mva-lyrics / mva-types / Engine 状态机 / manifest schema / CI workflow。

---

## 11. 测试策略

| 层 | 测试 | 位置 / 里程碑 |
|---|---|---|
| CLI 解析 (纯) | 无参→Empty; path→OpenProject; `--demo`→Demo; `--demo`+path→冲突; 未知 flag; `--help`/`--version` exit kinds | `cli.rs` `#[cfg(test)]` (M1) |
| Engine 失败语义 (Known Limitation 锁定) | A 播放中打开失败→Error + A 保留 + Play 拒绝 + Stop 恢复 | mva-core tests (M3) |
| 配置解析 | 查找顺序优先级 (temp dir); exe 目录优先于 CWD; 单文件损坏→该文件 fallback + warning 指出原因; 拼写错误 key→fallback + warning 含字段名; 双目录缺失→全默认 + warning | mva-core config tests (M4) |
| 启动加载 (集成, headless) | 以 fixtures 为主: `minimal.mva`→Ready + duration=manifest 值 + autoplay→`Audio(Play)` effect; prepare 失败 (缺失音频)→Error 映射; demo 序列→Playing + sine (无文件依赖) | `startup_flow.rs` (M5) |
| 真实资产冒烟 | `demo_showcase.rs` 维持 (examples 路径 + CC-BY mp3 依赖是有意且唯一的) | 现有, 角色重定位 |
| 设备失败通道 | 自动不可行 → 人工清单: 拔出/禁用设备→错误窗口可见 + stderr 有文本 + exit 1; 窗口系统不可用→stderr + exit 1 | 人工清单 (M6) |
| Empty/Demo 回归 | 无 project→Stopped/scene None (已有); `--demo` 行为与 v1 逐帧一致 | 现存 + M2 人工 |
| 存量套件 | `integration_flow` / `manifest_tests` / `engine_tests` / `phase3_*` 全绿 | 每里程碑 |
| CI | 无 workflow 变更 | — |

**原则**: 任何 crate 级测试不得唯一依赖 `../../examples` 相对路径或大型外部资产; fixtures 是默认夹具, examples 仅冒烟。

---

## 12. 里程碑

| # | 内容 | 验收 |
|---|---|---|
| **M1** 基础 | clap + `cli.rs` + 解析测试; **`impl Display for ProjectLoadError`** (必修前置); `dependencies.md` clap 条目; 退出码策略写入 phase4 文档 | 解析测试绿; Display 输出稳定文本 |
| **M2** 启动模式 | `startup.rs` bootstrap 区; main.rs 重写; Empty/Demo/OpenProject 三模式; 设备失败优雅终止通道 (错误窗口 + stderr + exit 1); 启动路径 `expect()` 清零; 窗口标题 | 无参 = idle; `--demo` = 旧行为; `<path>` 加载成功 (可手动 Play); 禁设备人工验证 |
| **M3** activate 统一 + autoplay | `activate_project` 两段式落地 + 锁契约 doc; UI OpenFile 改道共享 helper; `autoplay_on_open` 字段 + demo 豁免; §9.1 锁定测试; settings "Stop to reset" 提示; `Option<PathBuf>` 记录 | 两入口行为一致; autoplay=false 时两入口均不自动播; demo 仍自动播 |
| **M4** 配置系统 | §6.1 查找顺序 + 逐文件 fallback + warning 收集 (mva-core::config); settings 配置状态行 (MvaUiApp 参数); `deny_unknown_fields`; `app.toml` 更新 | 配置测试绿; 拼写错误 key→UI 可见 warning |
| **M5** fixtures + 集成测试 | 删 `test_project.rs`; `tests/fixtures/` 三件套; `startup_flow.rs`; phase2:613 与 architecture §13 更新 | 全 workspace 测试绿且无一依赖 examples 路径 (demo_showcase 除外) |
| **M6** 收尾 | help 文本; README/roadmap/project-status/CHANGELOG/examples README; 人工清单全项 (五命令 + 设备失败 + 配置 fallback 可见性); fmt/clippy/test | 文档与实现同步; 发布就绪 |

---

## 13. 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 运行期打开大文件冻结 UI 线程 | Medium | Medium | §9.2 如实记录; §3.3 两段式分界; async 化仅需 worker + completion dispatch, activate 复用 |
| 打开失败现场语义 (旧音频继续 + 只能 Stop 恢复) 触达真实用户 | Medium | Low | §9.1 记录 + UI 恢复提示 + 行为锁定测试; Engine transaction/revert 已列 Later |
| 配置路径解析在双击场景落空 | 已修复 | — | exe 目录优先 (§6.1) + 双目录缺失 warning 可见 |
| 配置拼写错误不可见 | 已修复 | — | `deny_unknown_fields` + settings 状态行; 降级场景 fallback+warning (已记录) |
| 设备错误窗口本身依赖窗口系统 | Low | Low | eframe 失败→stderr + exit 1 兜底 (§3.5) |
| fixtures 与格式演进漂移 | Low | Low | fixtures 每次 CI 被 startup_flow 执行; 格式变更立即变红 |
| examples 冒烟依赖 CC-BY mp3 | 接受 | Low | 有意保留单条; assets 已登记于 demo-assets.md |
| clap 编译重量 vs 3-flag CLI | 确定 | Low | derive-only; pico-args 备选已记录 |
| autoplay 打开第二个文件打断当前播放 | Medium | Low | 播放器惯例; `autoplay_on_open=false` 可关 |
| Windows 非 UTF-8 路径回归 | Low | Medium | clap PathBuf (OsStr) 全链路 |
| 范围蠕变 (拖拽/对话框/recent/headless) | Medium | Medium | WILL NOT 清单 + §4.4 演进策略收口 |

---

## 14. Decisions Requested

1. **设备失败通道**: 最小 eframe 错误窗口 + stderr + exit 1 (推荐) vs 仅 stderr + exit 1 (放弃双击场景可见性)。
2. **配置 I/O 归属**: `mva-core::config` (推荐, Editor 复用; Engine 保持零 I/O) vs startup.rs (接受未来复制)。
3. **`deny_unknown_fields`** 应用于 app 配置族 (推荐; pre-1.0 权衡已记录)。
4. **fixtures 提交入库** (phase2:613 原计划，推荐) vs 修订 phase2 文档取消 fixtures (不推荐 — 测试将长期依赖易变资产)。
5. **autoplay 策略**: `autoplay_on_open=true` 默认 + demo 豁免 + Editor 未来独立字段 (推荐维持)。
6. 批准后将本文档落盘为 `docs/phase4-architecture.md`, 随后按 M1→M6 顺序进入编码阶段。

---

**Revision v2 结论**: 原设计方向不变；审核指出的三处事实性/可行性缺陷 (设备承诺、Display 编译错误、fixtures 半计划) 与两处策略缺失 (配置路径解析、失败语义记录) 已全部修复，全部为文档级修订，唯一新增的代码面是 `Display` impl、最小错误窗口与 fixtures 文件 — 均在 Phase 4 既定范围内。审核通过后无需再次完整审核，可直接进入 M1。
