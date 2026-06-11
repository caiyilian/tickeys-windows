# Tickeys for Windows

> 打字音效反馈工具 - Windows 版本

![Tickeys Icon](https://raw.githubusercontent.com/yingDev/Tickeys/master/.readme_images/icon.png)

**Instant audio feedback for typing.** 为 Windows 设计的打字音效工具，提供多种音效方案，模拟机械键盘、打字机等声音。

## 特性

- ✅ 多种音效方案（机械键盘、打字机、鼓声、泡泡等）
- ✅ 支持自定义音效方案
- ✅ 应用黑白名单（指定哪些应用启用/禁用）
- ✅ 音量、音调调节
- ✅ 系统托盘运行
- ✅ 快捷键呼出设置界面（默认 `Q+A+Z+1+2+3`）

## 开发状态

🚧 **开发中** - Windows 版本正在重构中，使用 Rust + Win32 API 原生实现。

## 技术栈

- **语言**: Rust
- **GUI**: Win32 API (原生窗口控件)
- **键盘监听**: `SetWindowsHookEx(WH_KEYBOARD_LL)`
- **音频**: OpenAL-Soft
- **配置**: JSON 文件 + 注册表

## 开发环境

```bash
# 依赖
cargo build

# 运行
cargo run
```

## 许可证

MIT License - 继承自 [Tickeys](https://github.com/yingDev/Tickeys)

## 相关链接

- [Tickeys macOS 原版](https://github.com/yingDev/Tickeys)
- [Tickeys Linux 版](https://github.com/BillBillBillBill/Tickeys-linux)
