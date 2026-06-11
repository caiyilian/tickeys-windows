# Tickeys Windows 重构方案

> 将 Tickeys 从 macOS (Rust + Cocoa) 重构为 **Windows 原生版本** (Rust + Win32 API)

---

## 概述

### 项目背景

原项目 [Tickeys](https://github.com/yingDev/Tickeys) 是一个 macOS 打字音效反馈工具，使用 Rust + Cocoa/Objective-C 开发。本项目将其完全重写为 Windows 专用版本，使用 Win32 API 实现原生体验。

### 技术栈

| 层级     | 技术选型                           |
| -------- | ---------------------------------- |
| 语言     | Rust                               |
| GUI      | Win32 API (`windows` crate)        |
| 键盘监听 | `SetWindowsHookEx(WH_KEYBOARD_LL)` |
| 音频引擎 | OpenAL-Soft (`openal-sys`)         |
| 配置存储 | JSON 文件 + Windows 注册表         |
| 系统托盘 | `Shell_NotifyIconW`                |
| 构建工具 | Cargo + `winres`                   |

### 架构总览

```
┌─────────────────────────────────────────────────────────┐
│                    tickeys-windows                      │
├─────────────────────────────────────────────────────────┤
│  main.rs          ───  入口 + 消息循环 + 托盘           │
│  keyboard.rs      ───  全局键盘钩子                      │
│  audio.rs         ───  OpenAL 音频播放引擎               │
│  schemes.rs       ───  音效方案管理 (加载/切换)          │
│  gui.rs           ───  Win32 设置窗口                    │
│  config.rs        ───  配置存储 (JSON + 注册表)          │
│  filter.rs        ───  应用黑白名单过滤                   │
│  update.rs        ───  版本更新检查                      │
│  resource/        ───  音效文件 + 配置文件               │
│    data/schemes.json                                    │
│    data/*.wav                                           │
└─────────────────────────────────────────────────────────┘
```

---

## 阶段一：项目骨架与基础工程

> **目标**：搭建可编译运行的 Rust Win32 项目骨架

### 任务 1.1 — 创建 Cargo 项目

- 在 `tickeys-windows/` 中创建 `Cargo.toml`
- 引入 `windows` crate（含必备 feature）
- 引入必要依赖：`serde`、`serde_json`、`openal-sys`
- 创建 `build.rs`，嵌入 `.rs` 资源文件
- 配置 `.cargo/config.toml` 设置链接器选项（如有需要）

### 任务 1.2 — Win32 应用入口

- 实现 `WinMain` / `main` 入口
- 注册窗口类 (`RegisterClassW`)
- 创建隐藏主窗口（用于处理消息）
- 实现基础消息循环 (`GetMessageW` / `DispatchMessageW`)
- 验证：运行后无报错退出

### 任务 1.3 — 项目结构搭建

- 创建模块文件：`keyboard.rs`、`audio.rs`、`schemes.rs`、`gui.rs`、`config.rs`、`filter.rs`
- 在每个模块中写一个空的测试函数，确保编译通过
- 配置 `mod.rs` 或直接在 `main.rs` 中声明模块

### 里程碑 1

```
cargo run     # 弹出一个控制台窗口，输出 "Tickeys Windows started"
cargo build   # 编译无警告无错误
```

---

## 阶段二：键盘监听子系统

> **目标**：捕获全局键盘输入，提取键码

### 任务 2.1 — 低级键盘钩子实现

- 调用 `SetWindowsHookExW(WH_KEYBOARD_LL, ...)`
- 实现 `LowLevelKeyboardProc` 回调函数
- 提取 `KBDLLHOOKSTRUCT.vkCode`
- 将 keycode 通过 channel / callback 传递给主循环

### 任务 2.2 — 钩子生命周期管理

- 钩子的安装和卸载（`UnhookWindowsHookEx`）
- 程序退出时自动卸载钩子
- 异常处理：钩子安装失败时弹出错误提示

### 任务 2.3 — 防抖/频率限制

- 移植原版 `is_too_frequent` 逻辑
- 记录上次按键时间和键码
- 间隔小于 120ms 的相同按键跳过

### 里程碑 2

```
全局键盘钩子工作，在控制台打印每个按下的键码
```

---

## 阶段三：音频引擎

> **目标**：加载 WAV 文件并通过 OpenAL 播放

### 任务 3.1 — OpenAL 初始化

- 加载 `openal32.dll`（通过 `openal-sys` 或动态加载）
- 初始化 OpenAL 设备和上下文
- 错误处理：未找到 OpenAL 时给出用户提示

### 任务 3.2 — 音频数据加载

- 移植原版 `AudioData::from_file`
- 加载 WAV 文件为 OpenAL buffer (`alutCreateBufferFromFile`)
- 支持多音效文件的管理（Vec<AudioData>）

### 任务 3.3 — 音频播放

- 移植 `AudioSource` 结构体
- 移植 `SimpleAudioPlayer`（多源缓存、循环播放）
- 实现音量控制 (`alSourcef(AL_GAIN)`)
- 实现音调控制 (`alSourcef(AL_PITCH)`)
- 实现静音开关

### 任务 3.4 — 集成键盘 → 音效

- 将键盘钩子输出的 keycode → 映射为音效索引
- 支持 `key_audio_map` 精确映射
- 支持 `non_unique_count` 取模映射
- 通过 channel 将按键事件从钩子线程传递给主线程

### 里程碑 3

```
按键时播放对应音效（通过加载 resource/data/ 中的 WAV 文件）
```

---

## 阶段四：音效方案管理

> **目标**：读取 `schemes.json`，管理多套音效方案

### 任务 4.1 — 数据结构定义

- 移植 `AudioScheme` 结构体（含 `name`、`display_name`、`files`、`non_unique_count`、`key_audio_map`）
- 实现 `Deserialize` / `Serialize`

### 任务 4.2 — JSON 解析

- 读取 `resource/data/schemes.json`
- 反序列化为 `Vec<AudioScheme>`
- 错误处理：JSON 格式错误时给出错误提示

### 任务 4.3 — 方案切换

- `load_scheme(dir, name)` 方法
- 根据方案名查找对应配置
- 加载该方案的所有 WAV 文件
- 设置 keymap 映射规则

### 里程碑 4

```
支持多套音效方案加载，可在代码中硬编码切换方案测试
```

---

## 阶段五：配置持久化

> **目标**：保存/加载用户设置

### 任务 5.1 — 配置数据结构

```rust
struct Config {
    scheme: String,         // 当前音效方案名
    volume: f32,            // 音量 0.0 ~ 1.0
    pitch: f32,             // 音调 0.5 ~ 2.0
    filter_list: Vec<String>,  // 黑白名单应用列表
    filter_mode: FilterMode,   // BlackList | WhiteList
}
```

### 任务 5.2 — JSON 文件存储

- 配置文件路径：`%APPDATA%/Tickeys/config.json`
- 首次运行时创建默认配置
- 每次设置变更时自动保存
- 启动时加载配置

### 任务 5.3 — 注册表回退（可选）

- 以注册表 `HKEY_CURRENT_USER\Software\Tickeys` 作为备选存储
- 优先使用 JSON 文件，JSON 不存在时读取注册表

### 里程碑 5

```
修改配置后重启程序，配置保持不变
```

---

## 阶段六：应用黑白名单

> **目标**：指定哪些应用触发/不触发音效

### 任务 6.1 — 前台进程检测

- 通过 `GetForegroundWindow()` + `GetWindowThreadProcessId()` 获取前台进程
- 通过 `QueryFullProcessImageNameW()` 获取进程路径/名称
- 缓存进程名称以减少重复查询

### 任务 6.2 — 过滤逻辑

- 移植原版 `check_and_apply_mute_for_app` 逻辑
- 支持黑名单模式（列表中的应用静音）
- 支持白名单模式（仅列表中的应用发声）
- 窗口切换时自动更新静音状态

### 里程碑 6

```
在代码中硬编码测试白名单/黑名单，切换窗口时自动静音/取消静音
```

---

## 阶段七：GUI 设置窗口

> **目标**：完整的 Win32 设置界面

### 任务 7.1 — 窗口基础

- 创建设置窗口（模态对话框或普通窗口）
- 窗口标题："Tickeys 设置"
- 窗口置顶 (`WS_EX_TOPMOST`)
- 居中显示

### 任务 7.2 — 音效方案选择

- 下拉列表 (`ComboBox`) 显示所有方案
- 选择后立即切换方案
- 下拉项显示 `display_name`（支持中文）

### 任务 7.3 — 音量滑条

- `Trackbar` 控件：范围 0 ~ 100 (映射 0.0 ~ 1.0)
- 实时调整音量
- 显示当前音量百分比

### 任务 7.4 — 音调滑条

- `Trackbar` 控件：范围 50 ~ 200 (映射 0.5 ~ 2.0)
- 实时调整音调
- 显示当前音调值

### 任务 7.5 — 黑白名单管理

- 应用列表显示 (`ListView` 或 `ListBox`)
- "添加应用"按钮：弹出文件选择对话框，限制为 `.exe` 文件
- "移除选中"按钮
- 黑白名单模式切换 (`RadioButton` 或 `ComboBox`)
- 列表数据与 `config.rs` 联动

### 任务 7.6 — 窗口生命周期

- 显示/隐藏窗口（系统托盘控制）
- 窗口关闭时隐藏而非退出
- 窗口位置和状态保存

### 里程碑 7

```
完整 GUI 可用：选择方案、调节音量/音调、管理黑白名单
```

---

## 阶段八：系统托盘

> **目标**：最小化到系统托盘运行

### 任务 8.1 — 托盘图标创建

- `Shell_NotifyIconW(NIM_ADD, ...)`
- 设置托盘图标（从 `.ico` 文件或资源加载）
- 托盘 Tooltip 显示 "Tickeys"

### 任务 8.2 — 托盘菜单

- 右键菜单 (`TrackPopupMenu`)：
  - "显示设置"
  - "启用/禁用"（切换静音）
  - "退出"
- 左键双击：显示设置窗口

### 任务 8.3 — 托盘消息处理

- 处理 `WM_TRAYICON` 自定义消息
- 区分左键点击、右键点击、双击

### 里程碑 8

```
程序启动后隐藏在系统托盘，通过托盘图标交互
```

---

## 阶段九：快捷键呼出设置

> **目标**：通过特定按键序列呼出设置窗口

### 任务 9.1 — 按键序列检测

- 移植 `handle_keydown` 中的序列检测逻辑
- 记录最近 N 个按键（`VecDeque<u8>`）
- 匹配预设序列：`Q+A+Z+1+2+3` (键码 `12, 0, 6, 18, 19, 20`)

### 任务 9.2 — 呼出设置

- 匹配到序列时，显示设置窗口（若已隐藏）
- 若窗口已显示，则置前（`SetForegroundWindow`）

### 里程碑 9

```
按 QAZ123 呼出设置窗口
```

---

## 阶段十：发布准备与优化

> **目标**：打包为可分发版本

### 任务 10.1 — 应用图标与清单

- 嵌入 `.ico` 图标
- 创建 Windows 清单文件（支持 DPI 感知、管理员权限等）
- 设置 `winres` 构建脚本

### 任务 10.2 — 静默启动（无控制台）

- 将入口从 `console` 改为 `windows` 子系统
- 保留日志输出到文件（调试用）

### 任务 10.3 — 自启动

- 注册表 `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- 设置界面提供自启动选项

### 任务 10.4 — 版本检查更新

- 移植原版更新检查功能
- 启动时异步请求最新版本信息
- 有新版本时弹窗通知

### 任务 10.5 — 多语言支持

- 中/英文语言文件
- 根据系统语言自动切换

### 里程碑 10

```
发布第一个可用版本 (.exe 可分发)
```

---

## 阶段十一：高级特性与优化

> **目标**：提升用户体验

### 任务 11.1 — 粘滞键/筛选键检测

- 检测 Windows 粘滞键/筛选键状态
- 必要时提示用户关闭这些辅助功能

### 任务 11.2 — 音频设备热插拔

- 监听 `MMDEVAPI` 设备变更通知
- 音频设备变更时重新初始化 OpenAL

### 任务 11.3 — 音效预览

- 设置界面添加"试听"按钮
- 点击时播放对应方案的示例音效

### 任务 11.4 — 日志系统

- 文件日志（`%APPDATA%/Tickeys/log.txt`）
- 轮转日志，保留最近 3 个文件
- 便于用户反馈 Bug 时提供日志

### 里程碑 11

```
稳定版本，适合日常使用
```

---

## 阶段十二（可选）：安装包制作

> **目标**：生成安装程序

### 任务 12.1 — MSI 安装包

- 使用 `WiX Toolset` 或 `NSIS` 制作安装程序
- 包含 OpenAL 运行时分发

### 任务 12.2 — 自动更新

- 集成简单更新机制（下载新版本 → 替换 exe）
- 增量更新支持

---

## 附录

### A. 原项目代码映射

| 原文件                   | 功能                            | 新文件                            | 状态       |
| ------------------------ | ------------------------------- | --------------------------------- | ---------- |
| `src/main.rs`            | 应用入口、AppDelegate、更新检查 | `src/main.rs`                     | 重写       |
| `src/tickeys.rs`         | 音效播放核心、按键处理          | `src/audio.rs` + `src/schemes.rs` | 移植       |
| `src/event_tap.rs`       | CGEventTap 键盘监听             | `src/keyboard.rs`                 | 重写       |
| `src/settings_ui.rs`     | Cocoa 设置窗口                  | `src/gui.rs`                      | 重写       |
| `src/pref.rs`            | NSUserDefaults 配置             | `src/config.rs`                   | 重写       |
| `src/consts.rs`          | 常量定义                        | `src/consts.rs`                   | 移植       |
| `src/core_graphics.rs`   | CoreGraphics FFI 绑定           | —                                 | 删除       |
| `src/core_foundation.rs` | CoreFoundation FFI 绑定         | —                                 | 删除       |
| `src/alut.rs`            | alut + openal 绑定              | `src/audio.rs`                    | 保留并适配 |
| `src/cocoa_util.rs`      | Cocoa 工具函数                  | —                                 | 删除       |
| `data/schemes.json`      | 音效方案配置                    | `resource/data/schemes.json`      | 保留       |
| `data/*.wav`             | 音效文件                        | `resource/data/*.wav`             | 保留       |

### B. 原依赖替换对照

| 原依赖                             | 用途                  | 替换方案                     |
| ---------------------------------- | --------------------- | ---------------------------- |
| `cocoa`、`objc`、`block`           | Objective-C 绑定      | ❌ 删除                       |
| `core-foundation`、`core-graphics` | macOS 系统框架        | ❌ 删除                       |
| `IOKit-sys`                        | 电源事件              | `windows` crate (Power API)  |
| `openal-rs`                        | OpenAL 音频           | `openal-sys`                 |
| `hyper`                            | HTTP 请求（更新检查） | `windows` crate 或 `reqwest` |
| `rustc-serialize`                  | JSON 序列化           | `serde` + `serde_json`       |
| `time`                             | 时间函数              | `std::time`                  |
| `libc`                             | C 接口                | `windows` crate 或移除       |

### C. 命名规范

```
模块/文件：snake_case
结构体/枚举：PascalCase
函数/方法：snake_case
常量：SCREAMING_SNAKE_CASE
Windows API 调用：保持原命名风格（PascalCase）
```
