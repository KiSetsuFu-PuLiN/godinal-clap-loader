use crate::host::{
    message_processor::{PluginMessageProcessError, message_processor_impl::MessageProcessorImpl},
    plugin_message::PluginMessage,
};
use clack_host::{host::HostHandlers, plugin::PluginInstance};
use godot::{classes::Window, obj::Gd};
use std::cell::OnceCell;

/// 仅以命令行模式运行插件。
///
/// 注：虽然是这么说，但是实际用的时候发现还是有Gui显示，就很神奇。
pub struct Cli<T: HostHandlers> {
    plugin_instance: PluginInstance<T>,
}
impl<T: HostHandlers> Cli<T> {
    pub fn new(plugin_instance: PluginInstance<T>) -> Self {
        Self { plugin_instance }
    }
}
impl<T: HostHandlers> MessageProcessorImpl<T> for Cli<T> {
    fn plugin_instance(&self) -> &PluginInstance<T> {
        &self.plugin_instance
    }
    fn plugin_instance_mut(&mut self) -> &mut PluginInstance<T> {
        &mut self.plugin_instance
    }
    fn process(
        &mut self,
        host_shared_message: PluginMessage,
    ) -> Result<(), PluginMessageProcessError> {
        match host_shared_message {
            PluginMessage::RequestCallback => self.plugin_instance.call_on_main_thread_callback(),
            host_shared_message => return Err(host_shared_message.into()),
        };
        Ok(())
    }
    fn window(&self) -> Option<&OnceCell<Gd<Window>>> {
        None
    }
}
