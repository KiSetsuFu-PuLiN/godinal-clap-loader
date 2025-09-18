mod cli;
mod gui_embedded;
mod gui_floating;
pub mod message_processor_impl;

use crate::host::{
    Host,
    message_processor::message_processor_impl::{
        MessageProcessorImpl, new_plugin_message_processor_host,
    },
    plugin_message::{PluginGuiMessage, PluginMessage},
};
use clack_extensions::{
    gui::GuiError,
    log::{HostLogImpl, LogSeverity},
};
use clack_host::{host::HostHandlers, plugin::PluginInstance};
use std::{
    error::Error,
    fmt::Display,
    ops::{Deref, DerefMut},
    sync::mpsc::Receiver,
};

/// 插件消息处理器
pub struct MessageProcessor<T: HostHandlers> {
    /// 主机接受外部消息接收通道
    plugin_rx: Receiver<PluginMessage>,
    /// 主循环处理器
    message_processor_impl: Box<dyn MessageProcessorImpl<T>>,
}
impl MessageProcessor<Host> {
    pub fn new(plugin_rx: Receiver<PluginMessage>, plugin_instance: PluginInstance<Host>) -> Self {
        Self {
            plugin_rx,
            message_processor_impl: new_plugin_message_processor_host(plugin_instance),
        }
    }

    pub fn process(&mut self) {
        while let Ok(message) = self.plugin_rx.try_recv() {
            let message_process_result = self.message_processor_impl.process(message);
            if let Err(host_process_error) = message_process_result {
                self.plugin_instance().access_shared_handler(|host_shared| {
                    host_shared.log(
                        LogSeverity::HostMisbehaving,
                        &format!("{host_process_error}"),
                    )
                })
            }
        }
    }
}
impl<T: HostHandlers> Deref for MessageProcessor<T> {
    type Target = dyn MessageProcessorImpl<T>;
    fn deref(&self) -> &Self::Target {
        self.message_processor_impl.as_ref()
    }
}
impl<T: HostHandlers> DerefMut for MessageProcessor<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.message_processor_impl.as_mut()
    }
}

enum PluginMessageProcessorBuildError<T: HostHandlers> {
    GetGuiPreferredApiFailed {
        plugin_instance: PluginInstance<T>,
    },
    GuiError {
        plugin_instance: PluginInstance<T>,
        gui_error: GuiError,
    },
}
impl<T: HostHandlers> PluginMessageProcessorBuildError<T> {
    fn plugin_instance(self) -> PluginInstance<T> {
        match self {
            PluginMessageProcessorBuildError::GetGuiPreferredApiFailed { plugin_instance } => {
                plugin_instance
            }
            PluginMessageProcessorBuildError::GuiError {
                plugin_instance, ..
            } => plugin_instance,
        }
    }
}
impl<T: HostHandlers> Display for PluginMessageProcessorBuildError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginMessageProcessorBuildError::GetGuiPreferredApiFailed { .. } => {
                write!(f, "无法获取Gui推荐设置")
            }
            PluginMessageProcessorBuildError::GuiError { gui_error, .. } => {
                write!(f, "Gui显示异常：{}", gui_error)
            }
        }
    }
}

#[derive(Debug)]
pub enum PluginMessageProcessError {
    UnhandledMessage(PluginMessage),
    GuiError(GuiError),
    GodotWindowNotInitialized,
    GodotRootWindowNotFocused,
    GuiConfigurationNotFound,
    CannotOperateWhenGuiShowing(PluginGuiMessage),
    CannotOperateWhenGuiHideing(PluginGuiMessage),
}
impl Error for PluginMessageProcessError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PluginMessageProcessError::UnhandledMessage(..) => None,
            PluginMessageProcessError::GuiError(gui_error) => Some(gui_error),
            PluginMessageProcessError::GodotWindowNotInitialized => None,
            PluginMessageProcessError::GodotRootWindowNotFocused => None,
            PluginMessageProcessError::GuiConfigurationNotFound => None,
            PluginMessageProcessError::CannotOperateWhenGuiShowing(..) => None,
            PluginMessageProcessError::CannotOperateWhenGuiHideing(..) => None,
        }
    }
}
impl Display for PluginMessageProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginMessageProcessError::UnhandledMessage(host_shared_message) => {
                write!(f, "未处理的消息类型：{:?}", host_shared_message)
            }
            PluginMessageProcessError::GuiError(gui_error) => {
                write!(f, "操作插件GUI时错误：{gui_error}")
            }
            PluginMessageProcessError::GodotWindowNotInitialized => write!(
                f,
                "在Godot窗口还没有初始化的时候，调用了Godot窗口相关的方法"
            ),
            PluginMessageProcessError::GodotRootWindowNotFocused => write!(
                f,
                "Godot根窗口没有被聚焦，这会导致打开Godot的其他窗口失败。可能是Godot的嵌入调试模式导致的，因为在非嵌入模式下，根窗口总是聚焦的。这种情况下要么保持焦点不要脱离游戏窗口（尝试过通过代码在打开GUI前设置根窗口焦点，但是不起作用，这种情况下设置根窗口焦点本身就会导致abort panic），要么取消使用Godot嵌入调试功能。"
            ),
            PluginMessageProcessError::GuiConfigurationNotFound => write!(
                f,
                "无法从插件获取GuiConfiguration，可能是平台不支持显示GUI导致的。但是讲道理，如果平台不支持的话，应该以命令行模式加载插件才对"
            ),
            PluginMessageProcessError::CannotOperateWhenGuiShowing(plugin_gui_message) => {
                write!(f, "消息无法在Gui显示时执行：{:?}", plugin_gui_message)
            }
            PluginMessageProcessError::CannotOperateWhenGuiHideing(plugin_gui_message) => {
                write!(f, "消息无法在Gui隐藏时执行：{:?}", plugin_gui_message)
            }
        }
    }
}
impl From<PluginMessage> for PluginMessageProcessError {
    fn from(host_shared_message: PluginMessage) -> Self {
        Self::UnhandledMessage(host_shared_message)
    }
}
impl From<GuiError> for PluginMessageProcessError {
    fn from(gui_error: GuiError) -> Self {
        Self::GuiError(gui_error)
    }
}
