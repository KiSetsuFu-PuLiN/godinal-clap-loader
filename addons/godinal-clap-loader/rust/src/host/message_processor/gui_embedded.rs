use crate::host::{
    Host,
    message_processor::{
        PluginMessageProcessError, PluginMessageProcessorBuildError, cli::Cli,
        message_processor_impl::MessageProcessorImpl,
    },
    plugin_message::{PluginGuiMessage, PluginMessage},
};
use clack_extensions::gui::{
    GuiConfiguration, GuiSize, HostGuiImpl, PluginGui, Window as ClapWindow,
};
use clack_host::plugin::PluginInstance;
use godot::{
    classes::{DisplayServer, Window, display_server::HandleType},
    prelude::*,
    register::ConnectHandle,
};
use std::{cell::OnceCell, ffi::c_void};

/// 以Godot窗口运行插件
pub struct GuiEmbedded {
    host_processor_cli: Cli<Host>,
    plugin_gui: PluginGui,
    window: OnceCell<Gd<Window>>,
    window_signal_connections: Vec<ConnectHandle>,
}
impl GuiEmbedded {
    /// # Panic:
    /// 可能导致原因不明的系统冻结，尤其是在频繁变动窗口大小时。
    ///
    /// 疑似是使用[`DisplayServer::window_get_native_handle`]导致的。
    pub fn try_new(
        plugin_instance: PluginInstance<Host>,
        plugin_gui: PluginGui,
        // 不可去除，尽管构造时没用到，但是运行时还是会用。强制传参来保证这个配置存在。
        _gui_configuration: GuiConfiguration,
    ) -> Result<Self, PluginMessageProcessorBuildError<Host>> {
        // 使用命令行模式作为基础核心
        let host_processor_cli = Cli::new(plugin_instance);
        let host_processor_gui_embedded = Self {
            host_processor_cli,
            plugin_gui,
            window: OnceCell::new(),
            window_signal_connections: Vec::new(),
        };
        Ok(host_processor_gui_embedded)
    }

    /// 尝试获取Godot窗口。
    fn window(&self) -> Result<&Gd<Window>, PluginMessageProcessError> {
        self.window
            .get()
            .ok_or(PluginMessageProcessError::GodotWindowNotInitialized)
    }

    /// 尝试获取非根节点的jGodot窗口。
    fn window_with_no_root(&self) -> Result<Option<&Gd<Window>>, PluginMessageProcessError> {
        let window = self.window()?;
        let is_root = window
            .get_tree()
            .map(|scene_tree| scene_tree.get_root())
            .flatten()
            .map(|root| root == *window)
            .unwrap_or(false);
        Ok(if is_root { None } else { Some(window) })
    }

    /// 是否正在显示GUI
    fn is_showing(&self) -> bool {
        !self.window_signal_connections.is_empty()
    }

    /// 调整插件的显示大小与窗口适配
    fn adjust_plugin_gui_size(&mut self) -> Result<(), PluginMessageProcessError> {
        let size = self.window()?.get_size();
        let plugin_main_thread_handle = &mut self
            .host_processor_cli
            .plugin_instance_mut()
            .plugin_handle();
        self.plugin_gui.set_size(
            plugin_main_thread_handle,
            GuiSize {
                width: size.x as u32,
                height: size.y as u32,
            },
        )?;
        Ok(())
    }

    /// 设置窗口的大小。
    ///
    /// 注：在嵌入模式下无效，并被重定向到[`Self::adjust_plugin_gui_size`]。
    fn set_window_size(&mut self, gui_size: GuiSize) -> Result<(), PluginMessageProcessError> {
        // 在 Godot Debug 运行时，Godot 刚刚引入的嵌入调试窗口会导致无法设置窗口大小的bug。
        // 如果出现 "Embedded window can't be resized." 的报错，或者窗口大小并没有被成功设置的报错，
        // 关闭该嵌入功能即可修复。
        // https://github.com/godotengine/godot/issues/106496
        // https://www.reddit.com/r/godot/comments/1jj7hip/how_do_i_reenable_embed_game_on_next_play/
        let Some(window) = self.window_with_no_root()? else {
            return self.adjust_plugin_gui_size();
        };

        window.clone().set_size(Vector2i {
            x: gui_size.width as i32,
            y: gui_size.height as i32,
        });
        Ok(())
    }

    /// 显示窗口
    fn show(&mut self) -> Result<(), PluginMessageProcessError> {
        let mut plugin_main_thread_handle = self
            .host_processor_cli
            .plugin_instance_mut()
            .plugin_handle();
        let Some(mut gui_configuration) = self
            .plugin_gui
            .get_preferred_api(&mut plugin_main_thread_handle)
        else {
            return Err(PluginMessageProcessError::GuiConfigurationNotFound);
        };
        gui_configuration.is_floating = false;

        // 构造插件GUI。
        // 注：你可能觉得构造这句放在构造函数里比较好，但遗憾的是，那会导致在第二次打开这个窗口的时候界面不显示。
        self.plugin_gui
            .create(&mut plugin_main_thread_handle, gui_configuration)?;

        // Godot 窗口打开。
        let mut window = self.window()?.clone();
        let root = window
            .get_tree()
            .and_then(|scene_tree| scene_tree.get_root());
        if Some(window.clone()) != root {
            if let Some(initial_size) = self.plugin_gui.get_size(
                &mut self
                    .host_processor_cli
                    .plugin_instance_mut()
                    .plugin_handle(),
            ) {
                // 设置窗口初始大小。
                window.set_size(Vector2i {
                    x: initial_size.width as i32,
                    y: initial_size.height as i32,
                });
            }
            if let Some(root) = root
                // 在显示window之前，如果这个检查没过，就会发生不可逆的abort panic，详细请看返回的这个错误的说明。
                && !root.has_focus()
            {
                let mut plugin_main_thread_handle = self
                    .host_processor_cli
                    .plugin_instance_mut()
                    .plugin_handle();
                self.plugin_gui.destroy(&mut plugin_main_thread_handle);
                Err(PluginMessageProcessError::GodotRootWindowNotFocused)?;
            }
            window.show();
        }

        let mut plugin_main_thread_handle = self
            .host_processor_cli
            .plugin_instance_mut()
            .plugin_handle();
        let Some(api_type) = self
            .plugin_gui
            .get_preferred_api(&mut plugin_main_thread_handle)
            .map(|gui_configuration| gui_configuration.api_type)
        else {
            return Err(PluginMessageProcessError::GuiConfigurationNotFound);
        };

        // 附加到 Godot 窗口
        let generic_pointer = DisplayServer::singleton()
            .window_get_native_handle_ex(HandleType::WINDOW_HANDLE)
            .window_id(window.get_window_id())
            .done() as *mut c_void;
        // SAFETY: We ensure the window is valid for the lifetime of the plugin window.
        unsafe {
            self.plugin_gui.set_parent(
                &mut plugin_main_thread_handle,
                ClapWindow::from_generic_ptr(api_type, generic_pointer),
            )?
        };

        // 插件显示GUI。
        self.plugin_gui.show(&mut plugin_main_thread_handle)?;

        // 设置窗口事件
        let host_shared_size_changed = self
            .plugin_instance()
            .access_shared_handler(|host_shared| host_shared.clone());
        let host_shared_close_request = host_shared_size_changed.clone();
        self.window_signal_connections.push(
            window
                .signals()
                .size_changed()
                .connect(move || host_shared_size_changed.resize_hints_changed()),
        );
        self.window_signal_connections
            .push(window.signals().close_requested().connect(move || {
                host_shared_close_request.send(PluginMessage::Gui(PluginGuiMessage::RequestHide))
            }));

        Ok(())
    }

    /// 隐藏窗口
    fn hide(&mut self) -> Result<(), PluginMessageProcessError> {
        // 清空事件连接
        for connection in self.window_signal_connections.drain(..) {
            connection.disconnect();
        }

        // 插件隐藏GUI。
        self.plugin_gui.hide(
            &mut self
                .host_processor_cli
                .plugin_instance_mut()
                .plugin_handle(),
        )?;

        // Godot 窗口关闭。
        if let Some(mut window) = self.window_with_no_root()?.cloned() {
            // 这里有时会报错，类似PluginMessageProcessError::GodotRootWindowNotFocused，但是此处可以通过延迟进行规避。
            window.call_deferred("hide", &[]);
        }

        // 销毁插件GUI。
        self.plugin_gui.destroy(
            &mut self
                .host_processor_cli
                .plugin_instance_mut()
                .plugin_handle(),
        );

        Ok(())
    }

    fn process_when_showing(
        &mut self,
        gui_msg: PluginGuiMessage,
    ) -> Result<(), PluginMessageProcessError> {
        match gui_msg {
            PluginGuiMessage::ResizeHintsChanged => self.adjust_plugin_gui_size()?,
            PluginGuiMessage::RequestResize(gui_size) => self.set_window_size(gui_size)?,
            PluginGuiMessage::RequestHide => self.hide()?,
            PluginGuiMessage::RequestShow => Err(
                PluginMessageProcessError::CannotOperateWhenGuiShowing(gui_msg),
            )?,
        }
        Ok(())
    }
    fn process_when_hiding(
        &mut self,
        gui_msg: PluginGuiMessage,
    ) -> Result<(), PluginMessageProcessError> {
        match gui_msg {
            PluginGuiMessage::RequestShow => self.show()?,
            PluginGuiMessage::ResizeHintsChanged
            | PluginGuiMessage::RequestResize(_)
            | PluginGuiMessage::RequestHide => Err(
                PluginMessageProcessError::CannotOperateWhenGuiHideing(gui_msg),
            )?,
        }
        Ok(())
    }
}
impl MessageProcessorImpl<Host> for GuiEmbedded {
    fn plugin_instance(&self) -> &PluginInstance<Host> {
        self.host_processor_cli.plugin_instance()
    }
    fn plugin_instance_mut(&mut self) -> &mut PluginInstance<Host> {
        self.host_processor_cli.plugin_instance_mut()
    }
    fn process(
        &mut self,
        host_shared_message: PluginMessage,
    ) -> Result<(), PluginMessageProcessError> {
        match host_shared_message {
            PluginMessage::Gui(gui_msg) => {
                if self.is_showing() {
                    self.process_when_showing(gui_msg)?;
                } else {
                    self.process_when_hiding(gui_msg)?;
                }
            }
            msg => self.host_processor_cli.process(msg)?,
        };

        Ok(())
    }
    fn window(&self) -> Option<&OnceCell<Gd<Window>>> {
        Some(&self.window)
    }
}
impl Drop for GuiEmbedded {
    fn drop(&mut self) {
        for connection in self.window_signal_connections.drain(..) {
            if connection.is_connected() {
                connection.disconnect();
            }
        }
    }
}
