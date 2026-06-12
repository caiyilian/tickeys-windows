# Tickeys Windows 重构方案

> 将 Tickeys 从 macOS (Rust + Cocoa) 重构为 **Windows 原生版本** (Rust + Win32 API)

---

## 概述

### 项目背景

原项目 [Tickeys](https://github.com/yingDev/Tickeys) 是一个 macOS 打字音效反馈工具，使用 Rust + Cocoa/Objective-C 开发。本项目将其完全重写为 Windows 专用版本，使用 Win32 API 实现原生体验。

### 技术栈

| 层级     | 技术选型                              |
| -------- | ------------------------------------- |
| 语言     | Rust                                  |
| GUI      | Win32 API (`windows` crate)           |
| 键盘监听 | `SetWindowsHookExW(WH_KEYBOARD_LL)`   |
| 音频引擎 | OpenAL-Soft (`openal-sys` + 捆绑 DLL) |
| 配置存储 | JSON 文件 (`%APPDATA%/Tickeys/`)       |
| 系统托盘 | `Shell_NotifyIconW`                    |
| HTTP     | `reqwest` (异步更新检查)               |
| 构建工具 | Cargo + `winres`                      |

### 架构总览

```
┌─────────────────────────────────────────────────────────┐
│                    tickeys-windows                      │
├─────────────────────────────────────────────────────────┤
│  main.rs          ───  入口 + 消息循环 + 托盘           │
│  keyboard.rs      ───  全局键盘钩子 (WH_KEYBOARD_LL)    │
│  audio.rs         ───  OpenAL 音频播放引擎               │
│  schemes.rs       ───  音效方案管理 (加载/切换)          │
│  gui.rs           ───  Win32 设置窗口                    │
│  config.rs        ───  配置存储 (JSON)                   │
│  filter.rs        ───  应用黑白名单过滤                   │
│  update.rs        ───  版本更新检查                      │
│  consts.rs        ───  常量定义 (Windows VK 码)          │
│  tray.rs          ───  系统托盘管理                      │
│  power.rs         ───  电源事件监听                      │
│  log.rs           ───  日志系统                          │
│  resource/        ───  音效文件 + 配置文件               │
│    data/schemes.json                                    │
│    data/<scheme>/*.wav                                  │
│    icon.ico                                             │
└─────────────────────────────────────────────────────────┘
```

### 关键差异：macOS vs Windows 键码

原项目使用 macOS CGEvent 键码，Windows 使用 Virtual Key (VK) 码，**两者完全不同**：

| 按键 | macOS 键码 | Windows VK 码 | VK 常量名     |
| ---- | ---------- | ------------- | ------------- |
| Q    | 12         | 81 (0x51)     | `VK_Q`        |
| A    | 0          | 65 (0x41)     | `VK_A`        |
| Z    | 6          | 90 (0x5A)     | `VK_Z`        |
| 1    | 18         | 49 (0x31)     | `VK_1`        |
| 2    | 19         | 50 (0x32)     | `VK_2`        |
| 3    | 20         | 51 (0x33)     | `VK_3`        |
| Enter| 36         | 13 (0x0D)     | `VK_RETURN`   |
| Space| 49         | 32 (0x20)     | `VK_SPACE`    |
| Bksp | 51         | 8  (0x08)     | `VK_BACK`     |

**这意味着 `schemes.json` 中的 `key_audio_map` 也必须使用 Windows VK 码。** 有两种方案：
- **方案 A（推荐）**：为 Windows 单独维护一份 `schemes.json`，key_audio_map 使用 VK 码
- **方案 B**：在代码中维护一个 macOS→VK 码的映射表，运行时转换

---

## 阶段一：项目骨架与基础工程

> **目标**：搭建可编译运行的 Rust Win32 项目骨架

### 任务 1.1 — 创建 Cargo 项目

- 在 `tickeys-windows/` 中创建 `Cargo.toml`
- 引入 `windows` crate（含 `Win32_Foundation`、`Win32_UI_WindowsAndMessaging`、`Win32_System_LibraryLoader`、`Win32_UI_Input_KeyboardAndMouse` 等 feature）
- 引入依赖：`serde` + `serde_json`（配置序列化）、`openal-sys`（音频）、`reqwest`（HTTP）、`log` + `env_logger`（日志）
- 创建 `build.rs`，使用 `winres` 嵌入 `.ico` 图标和版本信息
- 创建 `resource/icon.ico`（从原版 `.icns` 转换或重新制作）

### 任务 1.2 — Win32 应用入口

- 实现 `WinMain` 入口（`#![windows_subsystem = "windows"]` 隐藏控制台）
- 注册窗口类 (`RegisterClassW`)，类名 `"TickeysMain"`
- 创建隐藏主窗口（`CreateWindowExW`，仅用于接收消息）
- 实现基础消息循环 (`GetMessageW` / `TranslateMessage` / `DispatchMessageW`)
- 窗口过程 (`WndProc`) 处理 `WM_DESTROY` → `PostQuitMessage`
- 验证：运行后无报错退出

### 任务 1.3 — 项目结构搭建

- 创建模块文件：`keyboard.rs`、`audio.rs`、`schemes.rs`、`gui.rs`、`config.rs`、`filter.rs`、`tray.rs`、`power.rs`、`consts.rs`、`log.rs`
- 在 `main.rs` 中声明所有模块
- 配置 `log` crate 输出到控制台（开发阶段）
- 验证：`cargo build` 无警告无错误

### 任务 1.4 — 资源文件准备

- 创建 `resource/data/` 目录
- 从 `Tickeys.app/Contents/Resources/data/` 复制所有音效子目录和 `schemes.json`
- 创建 Windows 版 `schemes.json`（修改 `key_audio_map` 使用 Windows VK 码）
- 创建 `resource/icon.ico`

### 里程碑 1

```
cargo run     # 弹出一个控制台窗口，输出 "Tickeys Windows started"
cargo build   # 编译无警告无错误
```

---

## 阶段二：常量定义与键码映射

> **目标**：定义所有 Windows VK 码常量和快捷键序列

### 任务 2.1 — Windows VK 码常量

```rust
// consts.rs
pub const VK_Q: u32 = 0x51;
pub const VK_A: u32 = 0x41;
pub const VK_Z: u32 = 0x5A;
pub const VK_1: u32 = 0x31;
pub const VK_2: u32 = 0x32;
pub const VK_3: u32 = 0x33;
pub const VK_RETURN: u32 = 0x0D;
pub const VK_SPACE: u32 = 0x20;
pub const VK_BACK: u32 = 0x08;
```

### 任务 2.2 — 快捷键序列定义

```rust
// 原版: [12, 0, 6, 18, 19, 20] (macOS: QAZ123)
// Windows: Q=81, A=65, Z=90, 1=49, 2=50, 3=51
pub const OPEN_SETTINGS_KEY_SEQ: &[&[u32]] = &[
    &[81, 65, 90, 49, 50, 51],  // QAZ123 (主键盘)
    &[81, 65, 90, 97, 98, 99],  // QAZ123 (数字键盘, VK_NUMPAD1-3)
];
```

### 任务 2.3 — 其他常量

```rust
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const WEBSITE: &str = "http://www.yingdev.com/projects/tickeys";
pub const DONATE_URL: &str = "http://www.yingdev.com/home/donate";
pub const APP_NAME: &str = "Tickeys";
pub const MUTEX_NAME: &str = "Global\\Tickeys_SingleInstance";
pub const WM_TRAYICON: u32 = WM_USER + 1;
```

### 里程碑 2

```
cargo build   # 编译通过，常量可引用
```

---

## 阶段三：键盘监听子系统

> **目标**：捕获全局键盘输入，提取键码

### 任务 3.1 — 低级键盘钩子实现

- 调用 `SetWindowsHookExW(WH_KEYBOARD_LL, Some(low_level_keyboard_proc), 0, 0)`
- 实现 `LowLevelKeyboardProc` 回调：
  ```rust
  unsafe extern "system" fn low_level_keyboard_proc(
      n_code: i32, w_param: WPARAM, l_param: LPARAM
  ) -> LRESULT
  ```
- 从 `KBDLLHOOKSTRUCT` 提取 `vkCode`
- **关键约束**：钩子回调中只做最小工作（发送消息/通过 channel 传递），不阻塞
- 通过 `PostMessageW` 将按键事件发送到主窗口

### 任务 3.2 — 钩子生命周期管理

- 钩子安装：在主窗口创建后安装
- 钩子卸载：`UnhookWindowsHookEx` 在程序退出前调用
- 弹窗处理：钩子安装失败时提示用户（可能需要管理员权限）
- **单实例检查**：使用命名 Mutex 防止多实例运行
  ```rust
  let mutex = CreateMutexW(None, true, MUTEX_NAME);
  if GetLastError() == ERROR_ALREADY_EXISTS {
      // 已有实例运行，提示并退出
  }
  ```

### 任务 3.3 — 防抖/频率限制

- 移植原版 `is_too_frequent` 逻辑
- 使用 `std::time::Instant` 替代原版 `time::precise_time_ns()`
- 间隔小于 120ms 的相同按键跳过
- 注意：需使用 `AtomicU64` 或 `Mutex` 保证线程安全（钩子回调在主线程）

### 任务 3.4 — 按键事件分发

- 主窗口 `WndProc` 处理自定义 `WM_KEYDOWN_HOOK` 消息
- 提取 vkCode 后调用 `tickeys.handle_keydown(vk_code)`
- 确保钩子回调和消息处理在同一消息循环中

### 里程碑 3

```
全局键盘钩子工作，在控制台打印每个按下的键码
按 QAZ123 能被检测到
```

---

## 阶段四：音频引擎

> **目标**：加载 WAV 文件并通过 OpenAL 播放

### 任务 4.1 — OpenAL 初始化

- 使用 `openal-sys` crate
- **捆绑 `openal32.dll`**：将 OpenAL-Soft 的 DLL 放在 exe 同目录
- 初始化：`alcOpenDevice` → `alcCreateContext` → `alcMakeContextCurrent`
- 错误处理：初始化失败时弹窗提示用户

### 任务 4.2 — 音频数据加载

- 移植 `AudioData` 结构体
- 使用 `alutCreateBufferFromFile` 加载 WAV 文件
- 管理多个 `AudioData` 的 `Vec`
- 错误处理：文件加载失败时记录日志

### 任务 4.3 — 音频播放器

- 移植 `AudioSource` 结构体（OpenAL source 管理）
- 移植 `SimpleAudioPlayer`（多源缓存、LRU 播放）
- 实现音量控制：`alSourcef(source, AL_GAIN, volume)`
- 实现音调控制：`alSourcef(source, AL_PITCH, pitch)`
- 实现静音开关（`mute` 标志位）

### 任务 4.4 — 集成键盘 → 音效

- 将键盘钩子输出的 vkCode → 映射为音效索引
- 支持 `key_audio_map` 精确映射
- 支持 `non_unique_count` 取模映射
- 注意：Windows VK 码范围与 macOS 不同，取模逻辑需验证

### 任务 4.5 — 动态重建播放器

- 当 `max_sources` 配置变更时，重建 `SimpleAudioPlayer`
- 实现 `SimpleAudioPlayer::rebuild(new_count)` 方法：
  - 释放旧的 OpenAL source（`alDeleteSources`）
  - 按新数量重新创建 source
  - 重新加载当前方案的音频数据
- 保持当前音量/音调设置不变
- 注意：重建期间短暂静音，用户几乎无感知

### 任务 4.6 — 资源路径解析

- Windows 无 macOS Bundle 结构
- 资源路径策略：exe 同目录下的 `data/` 文件夹
  ```rust
  fn get_resource_path(sub: &str) -> PathBuf {
      let exe = std::env::current_exe().unwrap();
      exe.parent().unwrap().join("data").join(sub)
  }
  ```
- 或使用 `%APPDATA%/Tickeys/data/`（用户可自定义）

### 里程碑 4

```
按键时播放对应音效（通过加载 resource/data/ 中的 WAV 文件）
```

---

## 阶段五：音效方案管理

> **目标**：读取 `schemes.json`，管理多套音效方案

### 任务 5.1 — 数据结构定义

```rust
#[derive(Deserialize, Serialize, Clone)]
pub struct AudioScheme {
    pub name: String,
    pub display_name: String,
    pub files: Vec<String>,
    pub non_unique_count: u8,
    pub key_audio_map: BTreeMap<u32, u8>,  // 注意: 使用 u32 (VK 码)
}
```

### 任务 5.2 — JSON 解析

- 读取 `data/schemes.json`
- 使用 `serde_json::from_str` 反序列化
- 错误处理：JSON 格式错误时记录日志并使用默认方案

### 任务 5.3 — 方案切换

- `load_scheme(dir, name)` 方法
- 根据方案名查找对应配置
- 加载该方案的所有 WAV 文件到 `AudioData`
- 设置 keymap 映射规则
- 卸载旧方案的音频数据

### 任务 5.4 — schemes.json Windows 版

原始 `schemes.json` 的 `key_audio_map` 使用 macOS 键码，需要转换为 Windows VK 码：

| 原 macOS 映射 | Windows VK 映射 |
| ------------- | --------------- |
| `"36": 8` (enter) | `"13": 8` |
| `"49": 5` (space) | `"32": 5` |
| `"51": 8` (backspace) | `"8": 8` |

### 里程碑 5

```
支持多套音效方案加载，可在代码中硬编码切换方案测试
```

---

## 阶段六：配置持久化

> **目标**：保存/加载用户设置

### 任务 6.1 — 配置数据结构

```rust
#[derive(Deserialize, Serialize)]
pub struct Config {
    pub scheme: String,              // 当前音效方案名
    pub volume: f32,                 // 音量 0.0 ~ 5.0（0%~500%，默认 0.5）
    pub pitch: f32,                  // 音调 0.5 ~ 2.0
    pub max_sources: usize,          // 同时播放音源数 1~8（默认 2，越大越少截断）
    pub filter_list: Vec<String>,    // 黑白名单应用列表 (exe 文件名)
    pub filter_mode: FilterMode,     // BlackList | WhiteList
    pub auto_start: bool,            // 开机自启动
}

#[derive(Deserialize, Serialize, PartialEq)]
pub enum FilterMode {
    BlackList,
    WhiteList,
}
```

### 任务 6.2 — JSON 文件存储

- 配置文件路径：`%APPDATA%/Tickeys/config.json`
- 首次运行时创建默认配置文件（`max_sources` 默认值为 2）
- 每次设置变更时自动保存（`config.save()`）
- 启动时加载配置（`Config::load()`）
- 配置目录自动创建：`std::fs::create_dir_all`

### 任务 6.3 — 配置验证

- `volume` 范围校验：`clamp(0.0, 5.0)`（支持 0%~500%）
- `pitch` 范围校验：`clamp(0.5, 2.0)`
- `max_sources` 范围校验：`clamp(1, 8)`，默认值 2
- `scheme` 存在性校验：不存在时回退到第一个方案
- `filter_list` 去重

### 里程碑 6

```
修改配置后重启程序，配置保持不变
```

---

## 阶段七：应用黑白名单

> **目标**：指定哪些应用触发/不触发音效

### 任务 7.1 — 前台进程检测

- 通过 `GetForegroundWindow()` 获取前台窗口句柄
- 通过 `GetWindowThreadProcessId()` 获取进程 ID
- 通过 `OpenProcess` + `QueryFullProcessImageNameW()` 获取进程完整路径
- 提取文件名（如 `chrome.exe`）
- 缓存进程名称，仅在窗口切换时更新

### 任务 7.2 — 窗口切换监听

- 使用 `SetWinEventHook` 监听 `EVENT_SYSTEM_FOREGROUND` 事件
- 事件触发时获取新前台进程名
- 调用过滤逻辑更新静音状态

### 任务 7.3 — 过滤逻辑

- 移植原版 `check_and_apply_mute_for_app` 逻辑
- 黑名单模式：列表中的应用 → 静音
- 白名单模式：列表中的应用 → 发声，其他 → 静音
- 窗口切换时自动更新静音状态

### 任务 7.4 — 权限处理

- 某些系统进程（如 `csrss.exe`、`winlogon.exe`）可能无法获取路径
- 对获取失败的进程，默认不静音（放行）

### 任务 7.5 — 按键黑名单

- `config.json` 新增 `blocked_keys: Vec<u16>` 字段
- 在 `map_key_to_audio()` 中检查：vk_code 在 blocked_keys 内 → 直接返回 None
- 可用于屏蔽特定按键（如音量键、功能键等）触发的音效
- 默认值为空列表

### 里程碑 7

```
在代码中硬编码测试白名单/黑名单，切换窗口时自动静音/取消静音
```

---

## 阶段八：系统托盘

> **目标**：最小化到系统托盘运行

### 任务 8.1 — 托盘图标创建

- `Shell_NotifyIconW(NIM_ADD, &nid)`
- 设置托盘图标（从 `resource/icon.ico` 加载）
- 托盘 Tooltip 显示 "Tickeys"
- 隐藏主窗口（仅托盘运行）

### 任务 8.2 — 托盘菜单

- 右键菜单 (`TrackPopupMenu`)：
  - "显示设置" → 显示设置窗口
  - "启用/禁用" → 切换全局静音
  - "退出" → 终止程序
- 左键双击：显示设置窗口

### 任务 8.3 — 托盘消息处理

- 在 `WndProc` 中处理 `WM_TRAYICON` 自定义消息
- `LOWORD(l_param)` 区分：`WM_LBUTTONDBLCLK`、`WM_RBUTTONUP`
- 菜单命令通过 `WM_COMMAND` 处理

### 任务 8.4 — 托盘图标更新

- 静音时切换为灰色图标
- 启用时切换为彩色图标
- `Shell_NotifyIconW(NIM_MODIFY, &nid)` 更新

### 里程碑 8

```
程序启动后隐藏在系统托盘，通过托盘图标交互
```

---

## 阶段九：GUI 设置窗口

> **目标**：完整的 Win32 设置界面

### 任务 9.1 — 窗口基础

- 创建设置窗口（`CreateWindowExW`）
- 窗口标题：`L"Tickeys 设置"`
- 窗口样式：`WS_OVERLAPPED | WS_CAPTION | WS_SYSBOX`
- 扩展样式：`WS_EX_TOPMOST`（置顶）
- 居中显示（`CenterWindow` 逻辑）
- 窗口关闭时隐藏而非退出（`WM_CLOSE` → `ShowWindow(SW_HIDE)`）

### 任务 9.2 — 音效方案选择

- 创建 `ComboBox` 控件
- 遍历所有方案，添加 `display_name` 到下拉列表
- 选择后立即切换方案（`CB_SELCHANGE` 通知）
- 加载时选中当前配置的方案

### 任务 9.3 — 音量滑条

- 创建 `Trackbar` 控件（`TRACKBAR_CLASSW`）
- 范围：0 ~ 500（映射 0.0 ~ 5.0，单位 %）
- 实时调整音量（`TB_THUMBTRACK` 通知）
- 旁边添加 `Static` 文本显示当前百分比（如 `"音量: 120%"`）
- 注意：超过 100% 会放大音量，可能产生削波失真，由用户自行决定

### 任务 9.4 — 音调滑条

- 创建 `Trackbar` 控件
- 范围：50 ~ 200（映射 0.5 ~ 2.0）
- 注意：原版 UI 中 1.0~1.5 映射为 1.0~2.0，Windows 版简化为线性映射
- 实时调整音调
- 显示当前音调值

### 任务 9.5 — 同时播放音源数

- 创建 `ComboBox` 控件，提供选项：1 / 2 / 3 / 4 / 5 / 6
- 默认选中 2（与原版 macOS 行为一致）
- 选择后立即生效：
  - 保存到 `config.max_sources`
  - 调用 `audio_player.rebuild(new_count)` 重建播放器
- 旁边添加 `Static` 文本说明："同时播放数（越大越不容易截断，占用资源略多）"
- 重启程序后保持用户选择

### 任务 9.6 — 黑白名单管理

- 创建 `ListView` 控件（`WC_LISTVIEWW`，`LVS_REPORT` 风格）
- 显示当前过滤列表
- "添加应用"按钮：
  - 弹出 `GetOpenFileNameW` 文件选择对话框
  - 过滤器：`"Applications (*.exe)\0*.exe\0"`
  - 支持多选
  - 去重后添加到列表
- "移除选中"按钮：删除选中项
- 黑白名单模式切换：两个 `RadioButton`（`"黑名单"` / `"白名单"`）
- 列表变更时自动保存配置

### 任务 9.7 — 版本显示

- 窗口底部 `Static` 文本显示 `"v0.5.0"`
- 可点击链接跳转到官网

### 任务 9.8 — 窗口位置管理

- 启动时居中显示
- 记住上次窗口位置（存入 config）
- 窗口始终置顶

### 里程碑 9

```
完整 GUI 可用：选择方案、调节音量/音调、设置同时播放数、管理黑白名单
```

---

## 阶段十：快捷键呼出设置

> **目标**：通过特定按键序列呼出设置窗口

### 任务 10.1 — 按键序列检测

- 移植 `handle_keydown` 中的序列检测逻辑
- 记录最近 N 个按键（`VecDeque<u32>`）
- 匹配预设序列：`Q+A+Z+1+2+3`（VK 码 `81, 65, 90, 49, 50, 51`）
- 从尾部向前比较

### 任务 10.2 — 呼出设置

- 匹配到序列时：
  - 若设置窗口已隐藏 → `ShowWindow(SW_SHOW)` + `SetForegroundWindow`
  - 若设置窗口已显示 → `SetForegroundWindow`（置前）
- 注意：`SetForegroundWindow` 可能被系统拒绝，需配合 `keybd_event` 技巧

### 里程碑 10

```
按 QAZ123 呼出设置窗口
```

---

## 阶段十一：电源事件与系统集成

> **目标**：处理系统电源事件，确保稳定运行

### 任务 11.1 — 电源事件监听

- 在 `WndProc` 中处理 `WM_POWERBROADCAST`
- `PBT_APMRESUMEAUTOMATIC` / `PBT_APMRESUMESUSPEND`：系统唤醒后重新初始化音频
- `PBT_APMSUSPEND`：系统休眠前清理资源

### 任务 11.2 — 音频设备变更

- 使用 `RegisterDeviceNotificationW` 监听音频设备变更
- 设备变更时重新初始化 OpenAL 上下文
- 或简化处理：每次播放前检查设备状态

### 任务 11.3 — DPI 感知

- 在 `main` 中调用 `SetProcessDpiAwareness(PROCESS_PER_MONITOR_DPI_AWARE)`
- 或在 manifest 文件中声明 DPI 感知
- 确保设置窗口在高 DPI 下正常显示

### 里程碑 11

```
系统休眠/唤醒后音频正常工作
```

---

## 阶段十二：版本更新检查

> **目标**：启动时异步检查新版本

### 任务 12.1 — HTTP 请求

- 使用 `reqwest` 发送异步 GET 请求
- URL：`http://www.yingdev.com/projects/latestVersion?product=Tickeys_0.4.0&lang=en-US`
- 启动后延迟 30 秒执行（避免影响启动速度）

### 任务 12.2 — 版本解析

```rust
#[derive(Deserialize)]
struct VersionInfo {
    Version: String,
    WhatsNew: String,
}
```

- 比较远程版本与 `CURRENT_VERSION`
- 版本不同时弹窗通知

### 任务 12.3 — 更新通知

- 使用 `MessageBoxW` 弹窗显示新版本信息
- 用户点击"确定"后打开浏览器访问下载页面
- 使用 `ShellExecuteW(None, "open", url, ...)` 打开浏览器

### 里程碑 12

```
启动 30 秒后检查更新，有新版本时弹窗提示
```

---

## 阶段十三：日志系统

> **目标**：便于调试和用户反馈

### 任务 13.1 — 文件日志

- 使用 `log` crate + `fern` 或 `simplelog`
- 日志文件路径：`%APPDATA%/Tickeys/log.txt`
- 格式：`[时间] [级别] 消息`
- 每次启动时清空旧日志（或轮转保留最近 3 个）

### 任务 13.2 — 开发/发布模式

- 开发模式：同时输出到控制台和文件
- 发布模式：仅输出到文件
- 通过 `#[cfg(debug_assertions)]` 切换

### 里程碑 13

```
日志文件正常写入，包含关键操作记录
```

---

## 阶段十四：发布准备与打包

> **目标**：打包为可分发版本

### 任务 14.1 — 应用图标

- 使用 `winres` 在 `build.rs` 中嵌入 `.ico` 图标
- 设置版本信息（`FILEVERSION`、`PRODUCTVERSION`）
- 设置公司名、产品名等

### 任务 14.2 — 清单文件

- 创建 `app.manifest`：
  - DPI 感知：`<dpiAwareness>PerMonitorV2</dpiAwareness>`
  - UAC 兼容：`<requestedExecutionLevel level="asInvoker" />`
  - Windows 版本兼容：`<supportedOS Id="{8e0f7a12-bfb3-4fe8-b9a5-48fd50a15a9a}"/>`

### 任务 14.3 — 单文件发布

- 将 exe、`openal32.dll`、`data/` 目录、`icon.ico` 打包为 zip
- 或使用 `cargo install --path .` 安装到用户目录

### 任务 14.4 — 自启动注册

- 提供自启动开关（GUI 中的 CheckBox）
- 注册表路径：`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- 键名：`"Tickeys"`
- 值：exe 完整路径

### 任务 14.5 — 安装包（可选）

- 使用 NSIS 或 WiX 制作安装程序
- 包含：exe、openal32.dll、data/ 目录
- 安装时自动注册自启动（可选）
- 卸载时清理注册表和配置文件

### 里程碑 14

```
发布第一个可用版本 (.exe 可分发)
```

---

## 附录

### A. 原项目代码映射

| 原文件                   | 功能                            | 新文件                            | 变更说明                           |
| ------------------------ | ------------------------------- | --------------------------------- | ---------------------------------- |
| `src/main.rs`            | 应用入口、AppDelegate、更新检查 | `src/main.rs`                     | 重写：WinMain + 消息循环 + 托盘    |
| `src/tickeys.rs`         | 音效播放核心、按键处理          | `src/audio.rs` + `src/schemes.rs` | 移植：OpenAL 部分保留，键码转换    |
| `src/event_tap.rs`       | CGEventTap 键盘监听             | `src/keyboard.rs`                 | 重写：WH_KEYBOARD_LL               |
| `src/settings_ui.rs`     | Cocoa 设置窗口                  | `src/gui.rs`                      | 重写：Win32 控件                   |
| `src/pref.rs`            | NSUserDefaults 配置             | `src/config.rs`                   | 重写：JSON 文件存储                |
| `src/consts.rs`          | 常量定义 (macOS 键码)           | `src/consts.rs`                   | 修改：所有键码改为 Windows VK 码   |
| `src/core_graphics.rs`   | CoreGraphics FFI 绑定           | —                                 | 删除                               |
| `src/core_foundation.rs` | CoreFoundation FFI 绑定         | —                                 | 删除                               |
| `src/alut.rs`            | alut + openal 绑定              | `src/audio.rs`                    | 保留并适配（openal-sys 替代）      |
| `src/cocoa_util.rs`      | Cocoa 工具函数                  | —                                 | 删除                               |
| `data/schemes.json`      | 音效方案配置 (macOS 键码)       | `resource/data/schemes.json`      | 修改：key_audio_map 改用 VK 码     |
| `data/*.wav`             | 音效文件                        | `resource/data/*.wav`             | 原样保留                           |

### B. 原依赖替换对照

| 原依赖                             | 用途                  | 替换方案                           |
| ---------------------------------- | --------------------- | ---------------------------------- |
| `cocoa`、`objc`、`block`           | Objective-C 绑定      | ❌ 删除                             |
| `core-foundation`、`core-graphics` | macOS 系统框架        | ❌ 删除                             |
| `IOKit-sys`                        | 电源事件              | `windows` crate (`WM_POWERBROADCAST`) |
| `openal-rs`                        | OpenAL 音频           | `openal-sys` + 捆绑 `openal32.dll` |
| `hyper`                            | HTTP 请求（更新检查） | `reqwest`                          |
| `rustc-serialize`                  | JSON 序列化           | `serde` + `serde_json`             |
| `time`                             | 时间函数              | `std::time::Instant`               |
| `libc`                             | C 接口                | `windows` crate 或移除             |

### C. 需要从原项目复制的资源

```
Tickeys.app/Contents/Resources/data/
├── bubble/
│   ├── 1.wav ~ 8.wav
│   └── enter.wav
├── typewriter/
│   ├── key-new-01.wav ~ key-new-05.wav
│   ├── space-new.wav
│   ├── scrollUp.wav
│   ├── scrollDown.wav
│   ├── backspace.wav
│   └── return-new.wav
├── mechanical/
│   └── 1.wav ~ 5.wav
├── sword/
│   ├── 1.wav ~ 6.wav
│   ├── back.wav
│   ├── enter.wav
│   └── space.wav
├── Cherry_G80_3000/
│   └── G80-3000*.wav (5 files)
├── Cherry_G80_3494/
│   └── G80-3494*.wav (6 files)
├── drum/
│   ├── 1.wav ~ 4.wav
│   ├── space.wav
│   ├── backspace.wav
│   └── enter.wav
└── schemes.json  (需修改 key_audio_map 为 VK 码)
```

### D. schemes.json 键码转换参考

原始 macOS 版本的特殊键映射：

| 方案          | macOS 映射                  | Windows VK 映射                 |
| ------------- | --------------------------- | ------------------------------- |
| bubble        | `"36":8` (enter)            | `"13":8` (VK_RETURN)            |
| typewriter    | `"36":9, "49":5, "51":8`   | `"13":9, "32":5, "8":8`         |
| mechanical    | `"36":4` (enter)            | `"13":4` (VK_RETURN)            |
| sword         | `"36":7, "49":8, "51":6`   | `"13":7, "32":8, "8":6`         |
| Cherry_G80_3000 | `"36":4, "49":4`         | `"13":4, "32":4`                |
| Cherry_G80_3494 | `"36":3, "49":4, "51":5` | `"13":3, "32":4, "8":5`         |
| drum          | `"36":6, "49":4, "51":5`   | `"13":6, "32":4, "8":5`         |

### E. 命名规范

```
模块/文件：snake_case
结构体/枚举：PascalCase
函数/方法：snake_case
常量：SCREAMING_SNAKE_CASE
Windows API 调用：保持原命名风格（PascalCase）
```

### F. 已知限制

1. **管理员权限进程**：`WH_KEYBOARD_LL` 无法捕获以管理员权限运行的应用的按键（除非 Tickeys 也以管理员运行）
2. **UWP 应用**：部分 UWP 应用的按键可能无法被捕获
3. **多显示器 DPI**：需要 Per Monitor DPI 支持
4. **音频设备热插拔**：需要额外处理，当前版本简化为重新初始化

---

## 开发顺序建议

```
阶段一 (骨架) → 阶段二 (常量) → 阶段三 (键盘) → 阶段四 (音频)
→ 阶段五 (方案) → 阶段六 (配置) → 阶段八 (托盘) → 阶段九 (GUI)
→ 阶段七 (黑白名单) → 阶段十 (快捷键) → 阶段十一 (电源)
→ 阶段十二 (更新) → 阶段十三 (日志) → 阶段十四 (打包)
```

推荐将 **系统托盘 (阶段八)** 提前到 **GUI (阶段九)** 之前，因为 GUI 需要通过托盘呼出。
