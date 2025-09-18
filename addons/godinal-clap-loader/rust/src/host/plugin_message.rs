use clack_extensions::gui::GuiSize;

/// 插件消息
#[derive(Debug)]
pub enum PluginMessage {
    RequestCallback,
    Gui(PluginGuiMessage),
}

/// 插件GUI消息
#[derive(Debug)]
pub enum PluginGuiMessage {
    ResizeHintsChanged,
    RequestResize(GuiSize),
    RequestShow,
    RequestHide,
}
