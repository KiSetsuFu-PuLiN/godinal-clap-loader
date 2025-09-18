use crate::host::plugin_message::{PluginGuiMessage, PluginMessage};
use clack_extensions::{
    gui::{GuiSize, HostGuiImpl},
    log::{HostLogImpl, LogSeverity},
    params::HostParamsImplShared,
};
use clack_host::host::{HostError, SharedHandler};
use godot::prelude::*;
use std::sync::mpsc::Sender;

/// 插件消息转发器，接受来自插件的回调，将其转换为[`通道消息`](PluginMessage)转发给主机，
/// 这样就可以让主机自行决定在什么合适的时机去处理这些消息。
#[derive(Clone)]
pub struct HostShared {
    plugin_tx: Sender<PluginMessage>,
}
impl HostShared {
    pub fn new(plugin_tx: Sender<PluginMessage>) -> Self {
        Self { plugin_tx }
    }

    pub fn send(&self, plugin_message: PluginMessage) {
        if let Err(err) = self.plugin_tx.send(plugin_message) {
            self.log(LogSeverity::HostMisbehaving, &format!("消息通道寄了，插件（消息）实例疑似已经被销毁了，HostShared不应该被带出到ClapPluginInstance实例之外：{err}"));
        }
    }
}
impl<'a> SharedHandler<'a> for HostShared {
    fn request_restart(&self) {
        todo!()
    }

    fn request_process(&self) {
        todo!()
    }

    fn request_callback(&self) {
        self.send(PluginMessage::RequestCallback);
    }
}
impl HostLogImpl for HostShared {
    fn log(&self, severity: LogSeverity, message: &str) {
        const HOST_LOG_NAME: &str = "Godinal Clap Loader Log";
        match severity {
            LogSeverity::Debug => godot_print!("{HOST_LOG_NAME} Debug: {message}"),
            LogSeverity::Info => godot_print!("{HOST_LOG_NAME} Info: {message}"),
            LogSeverity::Warning => godot_warn!("{HOST_LOG_NAME} Warning: {message}"),
            LogSeverity::HostMisbehaving => {
                godot_warn!("{HOST_LOG_NAME} HostMisbehaving: {message}")
            }
            LogSeverity::PluginMisbehaving => {
                godot_warn!("{HOST_LOG_NAME} PluginMisbehaving: {message}")
            }
            LogSeverity::Error => godot_error!("{HOST_LOG_NAME} Error: {message}"),
            LogSeverity::Fatal => godot_error!("{HOST_LOG_NAME} Fatal: {message}"),
        }
    }
}
impl HostGuiImpl for HostShared {
    fn resize_hints_changed(&self) {
        self.send(PluginMessage::Gui(PluginGuiMessage::ResizeHintsChanged));
    }

    fn request_resize(&self, new_size: GuiSize) -> Result<(), HostError> {
        self.plugin_tx
            .send(PluginMessage::Gui(PluginGuiMessage::RequestResize(
                new_size,
            )))?;
        Ok(())
    }

    fn request_show(&self) -> Result<(), HostError> {
        self.plugin_tx
            .send(PluginMessage::Gui(PluginGuiMessage::RequestShow))?;
        Ok(())
    }

    fn request_hide(&self) -> Result<(), HostError> {
        self.plugin_tx
            .send(PluginMessage::Gui(PluginGuiMessage::RequestHide))?;
        Ok(())
    }

    fn closed(&self, was_destroyed: bool) {
        todo!()
    }
}
impl HostParamsImplShared for HostShared {
    fn request_flush(&self) {
        todo!()
    }
}
