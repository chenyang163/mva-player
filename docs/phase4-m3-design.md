# MVA Phase 4 M3 Architecture Design — activate_project 统一加载管线 + autoplay 策略

**Status:** APPROVED
**Prerequisite:** Phase 4 M1/M2 已完成并通过审核
**基于事实:** 以下所有引用均已在当前源码中逐条核实 (`main.rs`, `startup.rs`, `app.rs`, `player.rs`, `state.rs`, `engine.rs`, `loader.rs`, `config/app.rs`, `settings.rs`)

---

## 1. Goal

将当前分散且部分行为不一致的两个加载入口统一为单一函数 `activate_project`，作为所有加载路径的唯一切入点。同时引入 `autoplay_on_open` 配置项，使自动播放行为在 CLI 路径与 UI 路径之间保持一致，且 `--demo` 获得显式豁免。

M3 不改动 Engine 状态机、不新增 command/enum/crate/service-layer。所有变更限定于 composition root 与配置结构体。

---

## 2. Current Problem

### 2.1 代码重复 (已核实)

**入口 A — CLI 启动加载** (`startup.rs:109-135` `boot_project`)：

```
loader.load(path) → Ok → 检查 ExternalFile → audio.load_file(audio_path)
                          → LoadProject (无 Play)
                    → Err → set_error(Unknown(e.to_string()))    ← 详细文本
```

**入口 B — UI 运行时加载** (`startup.rs:139-169` `build_on_effect` 的 `LoadProject` arm)：

```
loader.load(path) → Ok → 检查 ExternalFile → audio.load_file(audio_path)
                          → LoadProject (无 Play)
                    → Err → set_error(ProjectLoadFailed)        ← 丢失详情
```

两段 loader.load → ExternalFile 检查 → load_file → LoadProject 几乎逐行相同。仅错误映射不一致: 入口 A 传 `Unknown(Display 文本)`, 入口 B 传 `ProjectLoadFailed` (丢失原因细节)。

### 2.2 autoplay 未实现

两个入口当前均不执行 `Play`。按照 Phase 4 v2 设计 (§3.4), open = play 是对外播放器的一致行为。M3 必须在统一管线中引入 autoplay。

### 2.3 activate_project 为 stub

`startup.rs:189-197` 声明为 `#[allow(dead_code)] pub fn activate_project(...)`, 主体为空。M3 实现它并消除两个调用点的重复代码。

---

## 3. activate_project Design

### 3.1 签名与归属

```
// startup.rs runtime service zone

pub fn activate_project(
    path: PathBuf,
    engine: &Arc<Mutex<Engine>>,
    shared_audio: &SharedAudioPlayer,
    loader: &dyn ProjectLoader,
    autoplay: bool,                            // M3 新增
)
```

- **归属**: `crates/mva-player/src/startup.rs` runtime service 区域 (当前 stub 所在)。不抽新模块 — M2 文档已写明 *"规模增长时它是第一个被移出的候选"*, 当前规模可接受。
- **无返回值**: 所有成功/失败结果均通过 engine 状态反映 (set_error / LoadProject / Play), 调用方无需解释返回值。
- **不是管理对象**: 它是普通函数, 不持有状态, 不充当 service layer。

### 3.2 prepare 阶段

```
prepare(path, loader) → Result<(Project, Option<PathBuf>), ProjectLoadError>
```

| 步骤 | 操作 | 触碰对象 |
|---|---|---|
| 1 | `loader.load(&path)` → 解析 manifest / LRC / JSON / 目录探测 | 文件系统 |
| 2 | 提取 `project.audio.source` 的 `ExternalFile{path}` → 产出 `Option<PathBuf>` (音频来源路径) | 纯数据提取 |

失败 → 返回 `ProjectLoadError` (已有 `Display` 实现, M1 完成)。调用方经 `e.to_string()` 产出具语义的文本: "invalid manifest: bad JSON at line 5"。

注意: prepare 阶段不读取音频文件的内部数据 — `loader.load` 只解析 project 结构与引用路径。音频解码在 activate 的 `load_file` 中进行。

### 3.3 activate 阶段

```
activate(engine, shared_audio, project, audio_path, autoplay)
```

| 步骤 | 操作 | Engine 锁? | 失败后果 |
|---|---|---|---|
| 1 | 若 `audio_path.is_some()`: `shared_audio.load_file(&audio_path)` | 否 | `set_error(DecodeFailed)`; 旧 engine project 保留; 旧音频队列保留 (`player.rs:76-89` clear 只在解码成功后才执行) |
| 2 | `engine.lock()` → `handle_command(LoadProject(project))` | 是 | — (约不失败) |
| 3 | 若 `autoplay`: `handle_command(Play)` → 收集 `Vec<EngineEffect>` → 立即 drop lock → 遍历 effects, 对 `EngineEffect::Audio(cmd)` 调用 `shared_audio.apply(cmd)` | 是 (锁内执行 handle_command; lock 结束于 Play 返回值后、apply 循环前) | Play 命令本身不失败 (Read→Playing); `apply(Play)` 失败 → `eprintln! + 静默` (见 §6.2) |

### 3.4 调用方式 (两个入口收敛)

**入口 A — CLI 启动**:

```
main.rs → boot(OpenProject(path)) → boot_project → activate_project(path, engine, shared, &*loader, autoplay)
```

当前 `boot_project` 约 26 行替换为对 `activate_project` 的单次调用。内部不再有 loader.load / audio.load / LoadProject 逻辑。

**入口 B — UI 运行时**:

```
settings → PlayerCommand::OpenFile → Engine: Loading + LoadProject 效果
→ on_effect closure → activate_project(path, engine, shared, &*loader, autoplay)
```

当前 `build_on_effect` (140-169) 的 `LoadProject` 臂约 21 行替换为对 `activate_project` 的单次调用。`Audio` 臂不变, 保持现有 `let _ = shared_audio.apply(cmd)` 模式。

**两个入口的差异 (不消除)**:

| | CLI 启动 | UI 运行时 |
|---|---|---|
| Engine pre-state | Stopped (无项目) | Loading (OpenFile 命令设置) |
| 旧项目/旧音频 | 无 | 可能有 (Known Limitation §10.2) |
| 时序 | UI 未绘制 | egui 帧循环内 |

这些差异由引擎自身的行为决定 (OpenFile 命令的副作用), `activate_project` 不做特殊分支。

### 3.5 Demo 路径 — 显式不经过 activate_project

`--demo` 使用 `load_source(sine)` (内存源, 无文件) + `make_demo_project()` (程序化构建, 无 `loader.load`), 与 activate_project 的 prepare/activate 模式不兼容。保持 `startup.rs:60-94` 的 demo 启动序列不变 — 这是结构性免于 autoplay 门控: demo 代码路径从不询问 autoplay 变量, 因此天然不受配置影响。

---

## 4. Lock Contract

### 前提条件

调用方进入 `activate_project` 时不得持有 Engine 锁。

### 验证 — 两个调用点均已满足

| 调用点 | 锁状态 (已核实) |
|---|---|
| **入口 A (boot):** CLI 启动路径 | engine `Arc` 构造后尚未被任何代码持有; `boot_project` 在 `main.rs:45` 调用 `startup::boot()` 期间执行 — 此时无锁。 ✅ |
| **入口 B (on_effect):** UI 运行时路径 | `app.rs:49-83` 获取引擎锁、处理命令、收集 effects, `83` 行 `}; // engine lock dropped` 显式释放, `86-88` dispatch effects → on_effect → activate_project。锁已释放。 ✅ |

### 内部加锁规则 (activate 阶段)

```
engine.lock().unwrap()          ← 获取
    handle_command(LoadProject)  ← 在锁内
    if autoplay:
        handle_command(Play)     ← 在锁内
        // Play 返回的 effects 在锁内收集
// lock 在此作用域结束时释放        ← 释放 (lock guard dropped)
for effect in effects:           ← 锁外
    match Audio(cmd) => apply
```

音频效果 apply 在锁外执行，因为 `shared_audio.apply` 不触碰 Engine (`AudioController` trait 无 Engine 依赖), 且避免了长时间持锁 (audio.device sync 在 rodio 内部)。

### Deadlock 预防

- 系统中仅存在一个 Mutex (`Arc<Mutex<Engine>>`), 无多锁排序问题。
- `SharedAudioPlayer` 的 `apply`/`load_file` 不回调 Engine。
- `prepare` 阶段持零锁, 永远不会因为"持锁执行 I/O"而卡住。

---

## 5. autoplay Design

### 5.1 配置字段

新增于 `mva-core::config::app::GeneralConfig`:

```toml
# config/app.toml
[general]
autoplay_on_open = true
```

| 层级 | 变更 |
|---|---|
| `GeneralConfig` 结构体 | + `#[serde(default = "default_true")] pub autoplay_on_open: bool` |
| 辅助函数 (mva-core) | `fn default_true() -> bool { true }` |
| `AppConfig::default()` | + `autoplay_on_open: true` |
| `config/app.toml` | + `autoplay_on_open = true` |
| `mva-core/config_tests.rs` | 扩展现有 from_toml 测试: 含字段/缺字段默认 true/显式 false 解析; struct-literal 构造需补字段 (编译器报点) |

### 5.2 数据流与门控

```
main.rs → app_cfg.general.autoplay_on_open (bool) → startup::boot
         ↓                                     ↓
         boot_project                          build_on_effect 闭包捕获
              ↓                                     ↓
         activate_project(path, ..., autoplay) activate_project(path, ..., autoplay)
```

`--demo` 在 `boot` 内分支 (Enum → Demo), 不经过 autoplay 变量传递链, 结构性免除门控。

### 5.3 配置何时生效

M3 只加字段 + threading; `main.rs` 当前使用 `AppConfig::default()` (`main.rs:32`)。配置文件加载是 M4。因此 M3 实际运行时 autoplay 总是 `true` 默认值; `false` 行为可在 serde 往返测试中验证, 端到端验证随 M4 配置接线一起落地。M3 的自动化覆盖策略见 §9。

### 5.4 归属备注 (不实施)

该字段属于播放器关注点, 暂放共享 `GeneralConfig`。未来 Editor 有自己的配置文件 (与 `app.toml` 不同), 届时不受此字段影响。Phase 4 不拆 `[player]` 表 — 记录倾向。

---

## 6. Error Handling

### 6.1 错误矩阵

| 失败点 | 来源 | Engine 操作 | Engine 状态 | 旧 project | 旧音频 | 恢复 |
|---|---|---|---|---|---|---|
| prepare: loader.load 失败 | `ProjectLoadError` | `set_error(PlaybackError::Unknown(e.to_string()))` | Error | 保留 | 保持播放* | 按 Stop |
| prepare: loader.load 成功, 但非 ExternalFile 源 (未来内置音乐等) | — | 直接 LoadProject (无 load_file) → Ready | Ready | 替换为新 | silent (无音频源) | Play→apply 会失败? 取决于 player 是否有 source — 当前无 source 时 play 返回 NoSource。暂不处理: 当前所有 project 均为 ExternalFile |
| activate: audio.load_file 失败 | `AudioError` | `set_error(PlaybackError::DecodeFailed)` | Error | 保留** | 旧音频保持播放** | 按 Stop |
| activate: audio.apply(Play) 失败 | `CoreAudioError` | — (engine 已置 Playing) | Playing (空转) | 新 project | silent | stderr 可见** |

\* 仅运行时入口 B 可能出现"旧项目在播"场景 (已核实: `OpenFile` 命令不产生 `EngineEffect::Audio`, 即不暂停旧音频 — `engine.rs:136-140`)。CLI 启动入口 A 无此场景 (无旧项目)。

\*\* `load_file` 失败时旧 audio 队列完整保留 (已核实: `player.rs:76-89` 中 `File::open` 和 `Decoder::try_from` 失败时尚未执行 `player.clear()` at line 87 — clear 在解码成功后才执行)。

### 6.2 audio.apply 失败策略

`shared_audio.apply(Play)` 在激活阶段的 autoplay 中可能返回 `BackendError` (硬件断连等)。处理策略 (不需 Engine 感知):

1. `eprintln!("mva-player: autoplay audio failed: {e:?}")` — stderr 可见 (cargo run / 终端启动);
2. 静默吞下 — engine 状态已是 `Playing`, 每帧 poll 仍正确; position_seconds 返回实际进度; 用户可按 Pause/Play 重试 (apply_play 重新解码并播放 — `player.rs:282-293`);
3. 不需 `set_error`, 因为导致 engine 与 audio 状态不一致 (engine=Playing,audio=Stopped) 并不可恢复 — 用户手动 Stop+Play 是正确路径。

这在 `build_on_effect` 的 `Audio(cmd)` 臂中已有先例 — `let _ = shared_audio.apply(cmd)`。M3 沿用此模式。

### 6.3 入口 B 的错误映射统一

当前 M2 的入口 B (UI 路径) 在 loader.load 失败时映射为 `PlaybackError::ProjectLoadFailed` (丢失原因)。M3 统一调整为 `PlaybackError::Unknown(e.to_string())`, 与入口 A 对齐 — 这是 v2 文档 §3.3 的设计决定, 解决了"两个入口行为不一致"的问题。`ProjectLoadFailed` variant 保留在 enum 中供未来使用, Phase 4 不再消费它。

---

## 7. UI Integration

### 7.1 当前 UI 加载路径 (已核实, 不改变)

```
settings.rs:37 → PlayerCommand::OpenFile(path)         (输入)
app.rs:76-78  → engine.handle_command(cmd) → effects   (命令处理, 线程内, 持锁)
app.rs:82-83  → };  // engine lock dropped             (释放锁)
app.rs:86-88  → for effect in all_effects:             (dispatch, 锁外)
                    (self.on_effect)(effect)            ← 进入 activate_project
```

M3 不改变这个路径的架构。只改变 `on_effect` 闭包内 `LoadProject` 臂的函数体 — 从内联 21 行替换为调用 `activate_project`。

### 7.2 MvaUiApp 签名 (不改变)

`MvaUiApp::new` 的签名不变 (M4 会加 `config_warnings` 参数)。on_effect 闭包由 `startup::boot` 返回 (`main.rs:45`), 已在 composition-root 层构造, UI 不感知 autoplay/loader 细节。

### 7.3 settings 面板小改 — 恢复提示

在错误状态下 (`PlaybackState::Error`), settings 面板增加一行:

```
ui.colored_label(Color32::RED, format!("Error: {err:?}"));
ui.label("Press Stop to resume or open another file.");    // M3 新增
```

这是 v2 文档 §9.1 记录的 Known Limitation 对用户可见的表现 — 用户看到错误 + 唯一的恢复路径 (Stop) 提示。

---

## 8. File Changes

| 文件 | 变更 | 影响部位 |
|---|---|---|
| `crates/mva-player/src/startup.rs` | 实现 activate_project (替换 stub); `boot_project` 删除内联逻辑, 改为调用 `activate_project`; `build_on_effect` 的 `LoadProject` 臂替换为调用 `activate_project` 并消除 `ProjectLoadFailed` 错误映射; `boot` 签名增加 `autoplay: bool` 参数, 透传至 boot_project/build_on_effect | 约 +30 行 (实现) -50 行 (删除重复) → 净减 20 行 |
| `crates/mva-player/src/main.rs` | + `let autoplay = app_cfg.general.autoplay_on_open;` → 传入 `boot(..., autoplay)` | +2 行 |
| `crates/mva-core/src/config/app.rs` | `GeneralConfig` + `autoplay_on_open: bool` 字段 + serde 辅助函数 + `AppConfig::default()` 更新 + 无 `deny_unknown_fields` (M4) | +5 行 |
| `config/app.toml` | `[general]` + `autoplay_on_open = true` | +1 行 |
| `crates/mva-ui/src/panels/settings.rs` | Error 分支 + `"Press Stop to resume..."` 提示行 | +1 行 |
| `crates/mva-core/tests/config_tests.rs` | 扩展 autoplay 字段测试 (含字段/缺字段/显式 false); struct-literal 位置补字段 | +15 行 |
| `crates/mva-core/tests/engine_tests.rs` | 新增 Known Limitation 锁定测试: 停靠 A → 载入 B 失败 (set_error Unknown) → Error + A 保留 + Play 拒 + Stop 恢 | +25 行 |
| `crates/mva-player/tests/activate_flow.rs` | **NEW** — pipeline 序列测试 (无 UI, 走公开 crate): 两套序列 (autoplay=true → 有 Audio(Play) + Playing; autoplay=false → 无 Play + Ready); 基于 `examples/lyric_demo/` 资产。M5 迁移至 fixtures 后本文件改为 fixtures 主 + examples 冒烟 | +60 行 |

### 显式禁止修改 (M3 范围外)

| 禁改项 | 原因 |
|---|---|
| `mva-core/src/engine.rs`, `state.rs`, `command.rs`, `effect.rs` | Engine transaction 属于 Later |
| `mva-format/**` | loader 逻辑不变 |
| `mva-audio/**` | 不引入新方法/新 trait |
| `mva-ui/src/app.rs` | UI 循环不接触 activate_project |
| `crates/mva-player/src/cli.rs` | CLI surface 不变, 不新增 `--no-autoplay` 标志 |
| `mva-timeline/**`, `mva-scene/**`, `mva-renderer/**`, `mva-lyrics/**`, `mva-types/**` | 与 M3 正交 |

---

## 9. Test Plan

### 9.1 自动化 — 决策逻辑

| 测试 | 内容 | 文件 | 覆盖的 M3 需求 |
|---|---|---|---|
| serde 往返 | `toml: autoplay_on_open = true/false/缺失` — 正确解析; struct 构造补字段后不变 | `config_tests.rs` | autoplay 字段设计正确性 |
| Engine 失败语义锁定 | 构建 project A→LoadProject→Play(取效)→模拟"打开 B 失败"(手动 `set_error(Unknown)`)→**assert**: snap.state=Error, snap.project 为 A(scene=Some(..)), AudioCommand::Play 触发 Err(invalid_state), Stop→Stopped+error 清 | `engine_tests.rs` | Known Limitation 行为固化 |
| 管线序列 — autoplay=true | `MvaLoader::load(demo.mva)`→LoadProject→Play→assert: effects 含 `Audio(Play)`, state=Playing, duration=125.0 | `activate_flow.rs` (NEW) | autoplay 分支正确 |
| 管线序列 — autoplay=false | `MvaLoader::load(demo.mva)`→LoadProject→无 Play→assert: effects 空, state=Ready | `activate_flow.rs` (NEW) | autoplay=false 分支正确 |
| 管线序列 — prepare 失败 | `loader.load(nonexistent)`→error→`set_error(Unknown(text))`; snapshot Error; 验证 `PlaybackError::Unknown` 含文本 | `activate_flow.rs` (NEW) | 错误语义模块部分 |

`activate_flow.rs` 使用公共 crate (mva-timeline, mva-core, mva-format) 构建序列, 不调入 binary 私有函数。暂依赖 `examples/lyric_demo/demo.mva` (与现有 `demo_showcase.rs` 共享资产), M5 fixture 工作完成后迁移至 `tests/fixtures/minimal.mva`。

### 9.2 无法自动化的路径 (人工清单, M6)

- CLI autoplay=true 端到端: `cargo run -- examples/lyric_demo/demo.mva` → 自动开始播放
- CLI autoplay=false 端到端: 待 M4 配置加载接线后, 将 `app.toml` 设为 false 验证不自动播放
- UI OpenFile: 通过 UI 路径输入打开 → 自动播放行为与 CLI 一致
- demo 不受配置影响: `cargo run -- --demo` → 始终播放
- 设备失败通道 (禁设备): 错误窗口 → exit 1

### 9.3 存量回归

每里程碑运行 `cargo test --workspace` (现有 90+ 测试全绿) + `cargo clippy --workspace` + `cargo fmt --check`。

### 9.4 已知缺口 (诚实记录)

autoplay=false 的端到端路径在 M3 中无法自动化验证, 理由: M3 不加载配置文件 (M4 才接线), `AppConfig::default()` 始终为 `true`。M3 的覆盖是: 决策分支 (序列测试) + 字段解析 (config_tests) → 代码评审验证 wire。端到端验证随 M4/M6 完成。这是里程碑良性依赖 — M4 配置接线完成时, 所有相关代码已存在, 无需回头修改。

---

## 10. Known Limitations

### 10.1 引擎不区别"加载中但失败", 也没有 transaction barrier

Engine 第 7 态 (Error) 不分原因, 且不记录 "执行 LoadProject 前 engine 的状态"。接口层无法通过 engine 表面区分"这是打开失败"还是"这是播放中解码失败" — 对 UI 来说这些都是 Error。这是 Engine 设计尚未引入 transaction 的自然结果 (Phase 4 不实现), M3 不扩大这个限定。

### 10.2 打开 B 失败时旧 project A + 旧音频保持

见 §6.1 表格。唯一恢复路径是 Stop → Stopped → 再 OpenFile / Play。settings 面板已经在 Error 提示中加入了 Stop 路线 (§7.3)。

### 10.3 autoplay=false 在 M3 中仅可用默认值 true

配置文件加载在 M4。M3 期间 `autoplay_on_open` 对用户恒为 true。实现层面所有代码已准备就绪 — 只等 M4 配置接线。这在逻辑上是正确的里程碑依赖, 不需新增临时 CLI flag。

### 10.4 Phase 4 v2 §9 状态更新

Phase 4 v2 文档 §9 的已知限制继续有效; M3 对 §9.1 (失败语义) 首次提供了自动化行为测试, 将"已知"升级为"已知且测试锁定"。

---

## 11. Decisions

1. **`activate_project` 签名**: `autoplay: bool` vs `autoplay: GeneralConfig` (仅一个字段不配传整个配置结构体) — **推荐 bool**, 已采纳。
2. **入口 B 错误映射**: 统一用 `PlaybackError::Unknown(e.to_string())` 取代当前的 `ProjectLoadFailed` (解决不一致 → 详细错误文本)。`ProjectLoadFailed` variant 保留在 enum 中不删除。**已采纳。**
3. **activate_flow.rs 测试在 M3 依赖 examples 资产** (与现有 `demo_showcase.rs` 共享) — 已知是过渡性路径; M5 fixture 化时迁移。**已采纳, 测试注释中标明 `// M5: migrate to fixtures/minimal.mva`。**
4. **autoplay=false 端到端验证推迟至 M4**: 代码 wiring 已完成, 只差配置文件接线。**已采纳 M3-M4 里程碑依赖。**
5. **settings 面板 Error 提示改进**: `+ "Press Stop to resume or open another file."` — **已采纳 (1 行)。**
6. **`PlaybackError::Unknown` 显示格式**: settings 面板当前用 `Debug` 格式打印 — `"Unknown(\"invalid manifest: ...\")"` 略显机械但可读且不改状态结构体。**保留 Debug, 不新增 PlaybackError 的 Display impl。**

---

**文档状态: APPROVED — 可进入实现。**
