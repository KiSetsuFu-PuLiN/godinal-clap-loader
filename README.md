# ⚡ Godinal CLAP Loader

## 简介

**Godinal CLAP Loader** 是为 **Godot 引擎**量身打造的音频扩展，旨在为游戏和音频应用的开发引入 **[CLAP](https://github.com/free-audio/clap)** 插件格式的强大音频处理能力，将现代音频插件的**高性能和灵活性**带入 Godot 生态系统，让开发者能够在游戏内直接使用各种强大的 **CLAP 效果器和合成器**。

## ✨ 主要特性

- **音频流IO：** 提供一系列完整且低延迟的路由音频与 MIDI 数据流到 CLAP 插件的方法。
- **GUI显示：** 使用 Godot 原生窗口进行 CLAP 插件的 GUI 显示（如果插件有 GUI ）的话。
- **状态管理：** 支持 CLAP 插件实例状态的持久化存取，确保项目在重新加载或导出后，插件的音色和设置保持一致。
- **深度集成：** 使用 Godot 原生`AudioStream`和`InputEventMIDI`类型接口，提供近乎无感、操作简易的开发体验。

## 🚀 快速上手

1.  **克隆项目：** 将本项目根目录下的 [addons](addons) 文件夹克隆到您的项目中。您也可以通过 [Godot 资产库](https://example.com) 来进行这个操作。
2.  **运行示例：** 直接运行项目中的 [example.tscn](addons/godinal-clap-loader/example.tscn) 场景。Godot 会启动一个文件选择窗，用于选取一个本地的 CLAP 插件文件进行加载。
3.  **查阅文档：** 该场景会自动向加载的插件发送测试音频和控制信号。查看该场景的 GDScript 来了解如何使用插件常用的类，它们都附带有详细的文档注释！

💡 如果文字不够直观，还可以观看我录制的[使用教程](https://example.com)！

![](addons/godinal-clap-loader/Clap插件结构.drawio.svg)
*图：Clap插件结构抽象*

## 🛠️ 项目状态与开发计划

本项目核心功能已基本可用，但仍处于细节开发和完善阶段。

### 欢迎贡献

- 因为代码水平比较菜，有架构不合理或没测到的 Bug 大概是难免的（
- 而且还没怎么在项目里应用过，可能会漏掉一些实际用时会强烈需要的功能（
- 万一有发现，欢迎提 **Issue** 和 **Pull Request**

### 待办事项

以下是当前需要完善的几个主要方面：

1.  **代码清理：** 工程中的 `todo` 字段需要完善和整理。
2.  **跨平台支持：** 需要针对 Windows, macOS, Linux 和 Android（大概可以吧？） 等操作系统及各种硬件架构编译二进制库。
      -  编译后的文件将位于项目内的 [target](addons/godinal-clap-loader/rust/target) 文件夹，并通过[godinal-clap-loader.gdextension](addons/godinal-clap-loader/godinal-clap-loader.gdextension)被 Godot 项目引用并加载。
      -  这些版本需要更全面的测试和性能优化。

## 🔨 编译指南

如果 [target](addons/godinal-clap-loader/rust/target) 文件夹中没有您需要的目标平台二进制库，您需要自行编译：

1.  [安装 rust](https://www.rust-lang.org/learn/get-started)
2.  进入 [rust](addons/godinal-clap-loader/rust) 源代码目录
3.  运行以下命令来进行编译：
      - **调试版本：** `cargo build`
      - **发布版本：** `cargo build -r`

> 如果您成功编译了特定平台的二进制库，可以帮忙贡献到项目中，非常感谢🙏！

## 👏 谢鸣

本项目得以实现，离不开以下优秀项目和社区的支持与贡献：

- **[<img src="https://github.com/prokopyl/clack/blob/main/logo.svg" width="24" height="24"> clack](https://github.com/prokopyl/clack)**
- **[<img src="https://avatars.githubusercontent.com/u/66136469?s=24&v=4"> gdext](https://github.com/godot-rust/gdext)**
- **[<img src="https://avatars.githubusercontent.com/u/6318500?s=24&v=4"> Godot Engine](https://github.com/godotengine/godot)**
- **[<img src="https://avatars.githubusercontent.com/u/6681623?s=24&v=4"> Cardinal](https://github.com/DISTRHO/Cardinal)**

> 为什么本项目叫 Godinal Clap Loader ：
> > 这要感谢强大的模块化合成器 **[Cardinal](https://github.com/DISTRHO/Cardinal)** 在项目的初期作为了功能测试的主要实验对象，然后就拼凑出了 **Godinal** 这么个名字（雾
> >
> > 说不定以后哪天，这个项目真的可以成为一个名字大概叫 Godinal 的基于 Godot 的专业级 DAW 的一部分（

## 📜 许可证

本项目采用**混合许可证**。

1.  **项目主体代码：** 本仓库中由开发者编写的**主体代码**（不包括第三方依赖部分）采用 **MIT 许可证**发布。详情请参阅项目根目录下的 [LICENSE](LICENSE) 文件。
2.  **第三方依赖：** 本项目通过 [Cargo.toml](addons/godinal-clap-loader/rust/Cargo.toml) 引用了遵循 **MPL-2.0** 和其他许可证的库。在使用和分发本项目时，请务必遵守这些依赖库的相应许可证要求。

有关所有第三方依赖的完整许可证列表，请参阅 [LICENSES-THIRD-PARTY.md](LICENSES-THIRD-PARTY.md) 文件。
