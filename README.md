# ⚡ Godinal CLAP Loader

## 简介

**Godinal CLAP Loader** 是一个为 **Godot 引擎** 设计的扩展，旨在为游戏和音频应用引入 **CLAP (CLever Audio Plugin)** 插件格式的强大音频处理能力。

本项目将现代音频插件的灵活性和高性能带入 Godot 生态系统，允许开发者在游戏内使用专业级的 CLAP 效果器和合成器。

---

## ✨ 主要特性 (Features)

* **加载 CLAP 插件：** 轻松将 CLAP 插件文件集成到 Godot 项目中。
* **音频流路由：** 能够将 Godot 的 `AudioStream` 数据路由到 CLAP 插件的输入端口。
* **实时 MIDI 控制：** 支持在游戏运行时发送 MIDI 消息（如 Note On/Off、Control Change），以控制插件参数。
* **状态管理：** 能够加载和保存 CLAP 插件的内部状态，确保项目重启后效果一致。
* **跨平台支持：** 利用 Rust 的跨平台特性，为不同操作系统提供编译支持。

---

## 🛠️ 项目状态与开发计划

本项目核心功能已基本可用，但仍处于积极开发和完善细节阶段。

### 欢迎贡献 (Contributions Welcome!)

我们非常欢迎社区提交 **Issue** (反馈 Bug 或提出建议) 和 **Pull Request** (贡献代码)。您的反馈和帮助对于完善项目至关重要！

### 待办事项 (TODOs)

以下是当前需要完善的几个主要方面：

1.  **代码清理：** 工程中的 `TODO` 字段和临时注释需要完善和整理。
2.  **更全面的跨平台支持：** 当前 Rust 编译版本（位于 `addons/godinal-clap-loader/rust/target` 文件夹）需要针对 Windows, macOS, 和 Linux 等主流平台进行更全面的测试和优化。

---

## 🚀 快速上手 (Getting Started)

### 依赖与要求

* **Godot Version:** Godot 4.x (推荐最新稳定版)
* **系统环境:** 适用于 Godot 4.x 支持的任意操作系统。
* **插件文件:** 至少一个 CLAP 插件文件（通常为 `.clap` 扩展名）用于测试。

### 如何使用

1.  将 `godinal-clap-loader` 目录添加到您的 Godot 项目的 `addons/` 文件夹中。
2.  在 Godot 编辑器中，进入 **项目 > 项目设置 > 插件**，确保 **Godinal Clap Loader** 插件已启用。
3.  通过 GDScript 使用 `ClapPluginInstance.new_from_clap_files()` 方法加载你的 CLAP 插件。
4.  参考示例代码，设置音频输入输出，并开始发送 MIDI 消息控制插件。

---

## 许可证 (License)

本项目采用 **MIT 许可证** 开源。详情请见 [LICENSE](LICENSE) 文件。
