use crate::host::{
    message_processor::{
        PluginMessageProcessError, PluginMessageProcessorBuildError, cli::Cli,
        message_processor_impl::MessageProcessorImpl,
    },
    plugin_message::{PluginGuiMessage, PluginMessage},
};
use clack_extensions::gui::{
    GuiApiType, GuiConfiguration, GuiError, PluginGui, Window as ClapWindow,
};
use clack_host::{
    host::HostHandlers,
    plugin::{PluginInstance, PluginMainThreadHandle},
};
use godot::{
    classes::{DisplayServer, Engine, SceneTree, Window, display_server::HandleType},
    obj::Gd,
};
use std::{
    cell::OnceCell,
    ffi::{CString, c_void},
};

/// 以原生系统窗口运行插件，仅在[`支持的平台上`](clack_extensions::gui::GuiApiType::default_for_current_platform)有效。
///
/// 注：原生系统窗口在拖拽时会阻塞Godot。
pub struct GuiFloating<T: HostHandlers> {
    host_processor_cli: Cli<T>,
    plugin_gui: PluginGui,
}
impl<T: HostHandlers> GuiFloating<T> {
    pub fn try_new(
        mut plugin_instance: PluginInstance<T>,
        plugin_gui: PluginGui,
        gui_configuration: GuiConfiguration,
        transient: bool,
    ) -> Result<Self, PluginMessageProcessorBuildError<T>> {
        let mut plugin_main_thread_handle = plugin_instance.plugin_handle();

        // 创建窗口。
        let plugin_create_result =
            plugin_gui.create(&mut plugin_main_thread_handle, gui_configuration);
        if let Err(gui_error) = plugin_create_result {
            return Err(PluginMessageProcessorBuildError::GuiError {
                plugin_instance,
                gui_error,
            });
        }

        // 绑定窗口为Godot主窗口的子窗口（防止跑到主窗口的下面去）
        if transient {
            if let Err(gui_error) = set_transient(
                &plugin_gui,
                &mut plugin_main_thread_handle,
                gui_configuration.api_type,
            ) {
                return Err(PluginMessageProcessorBuildError::GuiError {
                    plugin_instance,
                    gui_error,
                });
            }
        }

        let mut host_processor_cli = Cli::new(plugin_instance);

        // 设置窗口标题。
        if let Ok(title) = CString::new(host_processor_cli.plugin_desc()) {
            let title = title.as_c_str();
            let mut plugin_main_thread_handle =
                host_processor_cli.plugin_instance_mut().plugin_handle();
            plugin_gui.suggest_title(&mut plugin_main_thread_handle, title);
        }

        // 使用命令行模式作为基础核心
        Ok(Self {
            host_processor_cli,
            plugin_gui,
        })
    }
}
impl<T: HostHandlers> MessageProcessorImpl<T> for GuiFloating<T> {
    fn plugin_instance(&self) -> &PluginInstance<T> {
        self.host_processor_cli.plugin_instance()
    }
    fn plugin_instance_mut(&mut self) -> &mut PluginInstance<T> {
        self.host_processor_cli.plugin_instance_mut()
    }
    fn process(
        &mut self,
        host_shared_message: PluginMessage,
    ) -> Result<(), PluginMessageProcessError> {
        match host_shared_message {
            PluginMessage::Gui(host_shared_gui_message) => match host_shared_gui_message {
                PluginGuiMessage::ResizeHintsChanged | PluginGuiMessage::RequestResize(..) => {
                    // 已被 PluginGui 处理
                }
                PluginGuiMessage::RequestShow => {
                    if let Some(main_loop) = Engine::singleton().get_main_loop()
                        && let Ok(scene_tree) = main_loop.try_cast::<SceneTree>()
                        && let Some(root) = scene_tree.get_root()
                        && !root.has_focus()
                    {
                        Err(PluginMessageProcessError::GodotRootWindowNotFocused)?
                    }
                    self.plugin_gui.show(
                        &mut self
                            .host_processor_cli
                            .plugin_instance_mut()
                            .plugin_handle(),
                    )?
                }
                PluginGuiMessage::RequestHide => self.plugin_gui.hide(
                    &mut self
                        .host_processor_cli
                        .plugin_instance_mut()
                        .plugin_handle(),
                )?,
            },
            msg => self.host_processor_cli.process(msg)?,
        };
        Ok(())
    }
    fn window(&self) -> Option<&OnceCell<Gd<Window>>> {
        None
    }
}
impl<T: HostHandlers> Drop for GuiFloating<T> {
    fn drop(&mut self) {
        let mut plugin_main_thread_handle = self
            .host_processor_cli
            .plugin_instance_mut()
            .plugin_handle();
        self.plugin_gui.destroy(&mut plugin_main_thread_handle);
    }
}

fn set_transient(
    plugin_gui: &PluginGui,
    plugin_main_thread_handle: &mut PluginMainThreadHandle,
    gui_api_type: GuiApiType,
) -> Result<(), GuiError> {
    let display_server = DisplayServer::singleton();
    let generic_pointer =
        display_server.window_get_native_handle(HandleType::WINDOW_HANDLE) as *mut c_void;
    // SAFETY: We ensure the window is valid for the lifetime of the plugin window.
    unsafe {
        plugin_gui.set_transient(
            plugin_main_thread_handle,
            ClapWindow::from_generic_ptr(gui_api_type, generic_pointer),
        )
    }
}
