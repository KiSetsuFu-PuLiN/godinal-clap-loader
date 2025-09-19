# ⚡ Godinal CLAP Loader

English | [简体中文](README.md)

## Introduction

**Godinal CLAP Loader** is an audio extension tailor-made for the **Godot Engine**, designed to introduce the powerful audio processing capabilities of the **[CLAP](https://github.com/free-audio/clap)** plugin format into game and audio application development. It brings the **high performance and flexibility** of modern audio plugins into the Godot ecosystem, allowing developers to directly use a variety of powerful **CLAP effects and synthesizers** within their games.

## ✨ Features

- **Audio Stream I/O:** Provides a complete set of low-latency methods for routing audio and MIDI data streams to CLAP plugins.
- **GUI Display:** Uses Godot's native windowing for displaying the CLAP plugin's GUI (if the plugin has one).
- **State Management:** Supports persistent saving and loading of the CLAP plugin instance state, which can be used to restore the plugin's patch and settings after it is reloaded.
- **Deep Integration:** Uses Godot's native `AudioStream` and `InputEventMIDI` type interfaces, offering a near-seamless and easy-to-use development experience.

## 🚀 Quick Start

1. **Clone the Project:** Clone the [addons](addons) folder from this project's root directory into your own project. You can also do this through the [Godot Asset Library](https://example.com).
2. **Run the Example:** Directly run the [example.tscn](addons/godinal-clap-loader/example.tscn) scene within the project. Godot will launch a file selector window for you to choose a local CLAP plugin file to load.
3. **Consult the Documentation:** The scene will automatically send test audio and control signals to the loaded plugin. Check the scene's GDScript to learn how to use the commonly utilized plugin classes, which all come with detailed documentation comments!

💡 If text isn't intuitive enough, you can also watch my recorded [Usage Tutorial](https://example.com)!

![](addons/godinal-clap-loader/Clap插件结构.drawio.svg)
*Figure: CLAP Plugin Structure Abstraction*

## 🛠️ Project Status and Development Plan

The core functionality of this project is largely usable but is still in the stage of detailed development and refinement.

### Contributions Welcome

- As my coding skills are relatively basic, architectural flaws or un-tested bugs are probably unavoidable (
- Also, since it hasn't been extensively applied in projects yet, some features that would be strongly needed in real-world use might be missing (
- If you find any issues, please feel free to submit **Issues** and **Pull Requests**.

### To-Do List

The following are several main areas that currently require improvement:

1. **Code Cleanup:** The `todo` fields in the project need to be completed and organized.
2. **Cross-Platform Support:** Binary libraries need to be compiled for operating systems such as Windows, macOS, Linux, and Android (perhaps?).
    - The compiled files will be located in the [target](addons/godinal-clap-loader/rust/target) folder within the project and referenced and loaded by the Godot project via [godinal-clap-loader.gdextension](addons/godinal-clap-loader/godinal-clap-loader.gdextension).
    - These versions require more comprehensive testing and performance optimization.

## 🔨 Compilation Guide

If the [target](addons/godinal-clap-loader/rust/target) folder does not contain the binary library for your required target platform, you will need to compile it yourself:

1. [Install Rust](https://www.rust-lang.org/learn/get-started).
2. Navigate to the [rust](addons/godinal-clap-loader/rust) source code directory.
3. Run the following commands to compile:
    - **Debug build:** `cargo build`
    - **Release build:** `cargo build -r`

> If you successfully compile the binary library for a specific platform, please consider contributing it to the project. Thank you very much 🙏!

## 👏 Acknowledgments

The realization of this project would not have been possible without the support and contributions of the following excellent projects and communities:

- **[<img src="https://github.com/prokopyl/clack/blob/main/logo.svg" width="24" height="24"> clack](https://github.com/prokopyl/clack)**
- **[<img src="https://avatars.githubusercontent.com/u/66136469?s=24&v=4"> gdext](https://github.com/godot-rust/gdext)**
- **[<img src="https://avatars.githubusercontent.com/u/6318500?s=24&v=4"> Godot Engine](https://github.com/godotengine/godot)**
- **[<img src="https://avatars.githubusercontent.com/u/6681623?s=24&v=4"> Cardinal](https://github.com/DISTRHO/Cardinal)**

> Why this project is named *Godinal Clap Loader*:
> > This is thanks to the powerful modular synthesizer **[Cardinal](https://github.com/DISTRHO/Cardinal)**, which served as the primary experimental subject for feature testing in the early stages of the project, and then the name **Godinal** was coined (a joke).
> >
> > Perhaps one day, this project could truly become a part of a professional-grade DAW based on Godot, probably named *Godinal* (

## 📜 License

This project uses a **Mixed License**.

1. **Main Project Code:** The **main code** written by the developer in this repository (excluding third-party dependencies) is released under the **MIT License**. For details, please see the [LICENSE](LICENSE) file in the project root.
2. **Third-Party Dependencies:** This project references libraries in [Cargo.toml](addons/godinal-clap-loader/rust/Cargo.toml) that follow **MPL-2.0** and other licenses. When using and distributing this project, you must comply with the respective license requirements of these dependent libraries.

For a complete list of licenses for all third-party dependencies, please refer to the [LICENSES-THIRD-PARTY.md](LICENSES-THIRD-PARTY.md) file.
