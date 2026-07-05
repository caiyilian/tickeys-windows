# Tickeys for Windows

> 打字音效反馈工具 — Windows 原生版本

**Instant audio feedback for typing.** 为 Windows 设计的打字音效工具，提供多种音效方案，模拟机械键盘、打字机、泡泡、鼓声等声音。本版本使用 Rust + Win32 API 原生实现，不依赖任何跨平台框架。

## 下载

👉 [最新版本 Releases](https://github.com/caiyilian/tickeys-windows/releases)

下载 `tickeys-windows-v*.zip`，解压后运行 `tickeys-windows.exe` 即可。

## 功能

### 音效

- **7 套音效方案**：bubble、typewriter、sword、mechanical、Cherry G80-3000、Cherry G80-3494、drum
- **音量 / 音调调节**：支持滑块实时调节
- **峰值同时播放数显示**：实时显示同时播放音源的最大数量，帮助用户合理设置同时播放数上限
- **长按仅触发一次**：按住按键不放只响一次，不会连续发声

### 按键控制

- **按键排除**：可自定义不发声的按键（默认排除 Backspace、Enter、Space、方向键、F1-F12）
- **按键防抖间隔可配置**：防止物理按键抖动导致重复触发，默认 20ms，范围 10-500ms，可在设置窗口实时调节
- **按键捕获添加**：在设置窗口中点击"添加按键"，按下任意键即可将其加入排除列表
- **应用黑白名单**：指定哪些应用启用或禁用音效

### 界面操作

- **系统托盘运行**：后台运行，托盘图标右键菜单可切换静音、打开设置、退出
- **快捷键呼出设置**：默认 `Q + A + Z + 1 + 2 + 3`（同时按下）
- **设置窗口**：Win32 原生窗口，支持所有功能调节
- **窗口位置记忆**：关闭后重新打开，窗口位置自动恢复
- **PerMonitorV2 DPI 感知**：在高 DPI 显示器上显示清晰

### 系统集成

- **电源事件监听**：休眠/唤醒后自动恢复音频
- **配置文件持久化**：设置保存在 `%APPDATA%/Tickeys/config.json`

## 设置窗口

按 `Q + A + Z + 1 + 2 + 3` 呼出设置窗口：

- **音效方案**：下拉选择音效方案
- **音量 / 音调**：滑块实时调节
- **同时播放数**：调节同时播放的音源数量上限（2-20），调低可减少资源占用
- **按键防抖(ms)**：调节同一按键防抖窗口（10-500ms），调低响应更灵敏
- **峰值同时播放数**：实时显示本次运行峰值
- **排除按键**：列表显示被排除的按键，支持添加/删除

## 自定义音效方案

1. 进入解压目录下的 `data/` 文件夹
2. 复制一个已有方案目录并重命名，例如 `drum` → `myDrum`
3. 编辑 `data/schemes.json`，复制对应的方案条目并修改 `name` 和 `display_name`
4. 替换 `.wav` 音频文件
5. 保存后重新打开设置窗口即可

schemes.json 格式说明：

| 字段 | 说明 |
|---|---|
| `name` | 目录名称，必须与文件夹名一致 |
| `display_name` | 界面显示名称 |
| `files` | 音频文件列表 |
| `non_unique_count` | 前 N 个文件自动映射到未指定的按键（取 `vkCode % non_unique_count`） |
| `key_audio_map` | 特定按键显式映射，格式为 `"虚拟键码": 音频索引` |

## 技术栈

- **语言**: Rust（2021 edition）
- **GUI**: Win32 API（原生窗口控件）
- **键盘监听**: `SetWindowsHookExW(WH_KEYBOARD_LL)` 全局低级键盘钩子
- **音频**: OpenAL-Soft（3D 音频库）
- **配置**: JSON 文件（`%APPDATA%/Tickeys/config.json`）
- **打包**: PowerShell 脚本打包为 zip 发布

## 开发环境

```bash
# 构建
cargo build

# 运行（开发模式显示控制台日志）
cargo run

# 发布构建
cargo build --release

# 打包
.\scripts\package-release.ps1
```

**注意**：发布时需要将 `resource/dll/OpenAL32.dll` 置于 exe 同级目录。

### 依赖项

- [OpenAL-Soft](https://openal-soft.org/) — 音频引擎（已内置 DLL）
- [windows](https://crates.io/crates/windows) — Windows API 绑定
- [serde](https://serde.rs/) — 配置序列化
- [hound](https://crates.io/crates/hound) — WAV 文件解析

## 发布历史

| 版本 | 日期 | 主要更新 |
|---|---|---|
| v1.0.3 | 2026-06-15 | 修复长按连续发声；mechanical 按键映射调整 |
| v1.0.2 | 2026-06-15 | 按键防抖间隔可配置（10-500ms） |
| v1.0.1 | 2026-06-13 | 新增峰值同时播放数显示 |
| v1.0.0 | 2026-06-13 | 首个可分发版本 |

## 许可证

MIT License — 继承自 [Tickeys](https://github.com/yingDev/Tickeys)

## 相关链接

- [Tickeys macOS 原版](https://github.com/yingDev/Tickeys)
- [Tickeys Linux 版](https://github.com/BillBillBillBill/Tickeys-linux)
- [Tickeys Windows（原版旧址）](https://www.yingdev.com/Content/Projects/Tickeys_Win/Release/1.1.1/Tickeys1.1.1.rar)