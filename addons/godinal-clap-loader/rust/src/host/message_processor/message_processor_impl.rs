use crate::host::{
    Host,
    message_processor::{
        PluginMessageProcessError, PluginMessageProcessorBuildError, cli::Cli,
        gui_embedded::GuiEmbedded, gui_floating::GuiFloating,
    },
    plugin_message::PluginMessage,
};
use clack_extensions::gui::{GuiConfiguration, PluginGui};
use clack_host::{host::HostHandlers, plugin::PluginInstance};
use godot::{classes::Window, prelude::*};
use std::{cell::OnceCell, ffi::CStr};

/// 插件消息处理器核心特征
pub trait MessageProcessorImpl<T: HostHandlers> {
    /// 访问Clap插件实例。
    fn plugin_instance(&self) -> &PluginInstance<T>;
    /// 访问Clap插件实例。
    fn plugin_instance_mut(&mut self) -> &mut PluginInstance<T>;
    /// 处理主机消息的，需要一直调用的函数。
    fn process(
        &mut self,
        host_shared_message: PluginMessage,
    ) -> Result<(), PluginMessageProcessError>;
    /// 如果不为空，说明这个Clap插件需要Godot窗口的支持。
    /// 在Godot端进行初始化。
    fn window(&self) -> Option<&OnceCell<Gd<Window>>>;

    /// 插件描述
    fn plugin_desc(&self) -> String {
        unsafe {
            let desc = *self.plugin_instance().raw_instance().desc;
            format!(
                "{} - {}",
                CStr::from_ptr(desc.name).to_string_lossy(),
                CStr::from_ptr(desc.id).to_string_lossy()
            )
        }
    }
}

fn log_head_name(name: &str) -> String {
    format!("ClapGui：{name}")
}
fn log_build_succ(name: &str) {
    godot_print!("{}", log_head_name(name))
}
fn log_build_err<T: HostHandlers>(name: &str, err: &PluginMessageProcessorBuildError<T>) {
    godot_warn!(
        "构造失败：{}：{}，将尝试备用方案。",
        log_head_name(name),
        err
    );
}
fn cli(plugin_instance: PluginInstance<Host>) -> Box<dyn MessageProcessorImpl<Host>> {
    log_build_succ("命令行");
    Box::new(Cli::new(plugin_instance))
}
fn floating_gui(
    plugin_instance: PluginInstance<Host>,
    plugin_gui: PluginGui,
    mut gui_configuration: GuiConfiguration,
) -> Box<dyn MessageProcessorImpl<Host>> {
    let name = "系统浮动窗口";
    gui_configuration.is_floating = true;
    match GuiFloating::try_new(plugin_instance, plugin_gui, gui_configuration, false) {
        Ok(floating_gui) => {
            log_build_succ(name);
            Box::new(floating_gui)
        }
        Err(err) => {
            log_build_err(name, &err);
            let plugin_instance = err.plugin_instance();
            cli(plugin_instance)
        }
    }
}
fn embedded_gui(
    plugin_instance: PluginInstance<Host>,
    plugin_gui: PluginGui,
    mut gui_configuration: GuiConfiguration,
) -> Box<dyn MessageProcessorImpl<Host>> {
    let name = "Godot浮动窗口";
    gui_configuration.is_floating = false;
    match GuiEmbedded::try_new(plugin_instance, plugin_gui, gui_configuration) {
        Ok(embedded_gui) => {
            log_build_succ(name);
            Box::new(embedded_gui)
        }
        Err(err) => {
            log_build_err(name, &err);
            floating_gui(err.plugin_instance(), plugin_gui, gui_configuration)
        }
    }
}
pub fn new_plugin_message_processor_host(
    mut plugin_instance: PluginInstance<Host>,
) -> Box<dyn MessageProcessorImpl<Host>> {
    match plugin_instance
        .access_handler(|host_main_thread| host_main_thread.get_extension::<PluginGui>())
    {
        Some(plugin_gui) => {
            match plugin_gui.get_preferred_api(&mut plugin_instance.plugin_handle()) {
                Some(gui_configuration) => {
                    embedded_gui(plugin_instance, plugin_gui, gui_configuration)
                }
                None => {
                    let err = PluginMessageProcessorBuildError::GetGuiPreferredApiFailed {
                        plugin_instance,
                    };
                    log_build_err("Gui基础环境", &err);
                    cli(err.plugin_instance())
                }
            }
        }
        None => cli(plugin_instance),
    }
}
