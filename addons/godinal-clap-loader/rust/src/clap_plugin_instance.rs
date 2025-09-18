use crate::{
    clap_input_audio_port_access::ClapInputAudioPortAccess,
    clap_output_audio_port_access::ClapOutputAudioPortAccess,
    clap_transport_event_access::ClapTransportEventAccess,
    host::{Host, host_handlers_impl::host_shared::HostShared},
    midi::{event_to_midi, midi_to_event},
};
use clack_extensions::{
    gui::HostGuiImpl,
    log::{HostLogImpl, LogSeverity},
};
use godot::{
    classes::{
        DisplayServer, InputEventMidi, Window, display_server::WindowMode, node::InternalMode,
        notify::NodeNotification, window::WindowInitialPosition,
    },
    prelude::*,
};
use std::path::Path;

/// Clap插件实例在Godot端的代理。
#[derive(GodotClass)]
#[class(no_init, base = Node)]
struct ClapPluginInstance {
    #[base]
    base: Base<Node>,

    host: Host,

    /// 为插件设置的当前宿主数字音频工作站(DAW)的播放传输状态信息，用于同步插件处理与宿主播放时间轴。
    #[allow(unused)]
    #[var(get=get_clap_transport_event_access, set = set_clap_transport_event_access)]
    clap_transport_event_access: Option<Gd<ClapTransportEventAccess>>,

    /// 插件当前的状态，可以取出宿主的这个状态并持久化，方便后续打开这个重新加载这个宿主时快速从持久化数据恢复插件设置。
    #[allow(unused)]
    #[var(get = get_state, set = set_state)]
    state: PackedByteArray,
}
impl ClapPluginInstance {
    /// 构造
    fn new(host: Host) -> Gd<Self> {
        let mut clap_plugin_instance = Gd::from_init_fn(|base| Self {
            base,
            host,
            clap_transport_event_access: None,
            state: PackedByteArray::new(),
        });
        clap_plugin_instance.set_process_internal(true);
        clap_plugin_instance
    }
    /// 初始化Clap插件的Godot窗口
    fn init_window(&mut self) {
        if self.host.message_processor().window().is_none() {
            self.show_gui();
            return;
        }

        let display_server = DisplayServer::singleton();
        let window = if display_server.window_get_mode() == WindowMode::EXCLUSIVE_FULLSCREEN {
            // 无法使用多窗口的情况，就用Root作为插件的显示window。
            let Some(window) = self
                .base()
                .get_tree()
                .map(|scene_tree| scene_tree.get_root())
                .flatten()
            else {
                self.log(
                    LogSeverity::HostMisbehaving,
                    "初始化Clap插件的Godot窗口失败：找不到Root窗口",
                );
                return;
            };
            window
        } else {
            let mut window = Window::new_alloc();
            window.hide();
            window.set_size(display_server.window_get_size());
            window.set_initial_position(WindowInitialPosition::CENTER_MAIN_WINDOW_SCREEN);
            window.set_title(&self.host.message_processor().plugin_desc());
            self.base_mut()
                .add_child_ex(&window)
                .internal(InternalMode::FRONT)
                .done();
            window
        };

        let Some(once_cell_window) = self.host.message_processor().window() else {
            self.log(
                LogSeverity::HostMisbehaving,
                &format!("初始化Clap插件的Godot窗口失败：插件的Godot窗口需求性质刚刚发生了变化"),
            );
            return;
        };
        if let Err(err) = once_cell_window.set(window) {
            self.log(
                LogSeverity::HostMisbehaving,
                &format!("初始化Clap插件的Godot窗口失败：疑似被初始化了多次：{err}"),
            );
            return;
        }

        self.show_gui();
    }

    /// 插件消息句柄，用于向插件发送指令
    fn host_shared(&self) -> &HostShared {
        self.host
            .message_processor()
            .plugin_instance()
            .access_shared_handler(|host_shared| host_shared)
    }
    /// 进行日志打印
    fn log(&self, severity: LogSeverity, message: &str) {
        self.host_shared().log(severity, message)
    }
}
#[godot_api]
impl ClapPluginInstance {
    /// 通过文件路径，加载内部包含的所有Clap插件。
    /// - `path`: clap插件文件系统路径。
    /// - `sample_rate`: 采样率，决定了本处理实例读取和写入音频缓冲的整体帧速率。
    /// - `max_latency_seconds`: 最大延迟秒数。代表着Clap插件处理完音频信号之后，这些输出信号最多会在缓存里面留多长时间。用这个参数乘上采样率就是缓冲区的最大帧长度。[color=yellow]注意：这个值不要过小，否则clap插件线程数据的读取线程遭遇帧率扰动时会容易导致卡顿。如果这个值小于读取线程的最小帧间隔（指`_process`的`delta`），则会无法正常读取音频数据。[/color][color=red]也不要太大，内存会炸的。[/color]
    ///
    /// 返回：加载的各个clap实例。（一个clap文件可能包含多个实例）
    #[func]
    fn new_from_clap_file(
        path: GString,
        sample_rate: f64,
        max_latency_seconds: f64,
    ) -> Array<Gd<Self>> {
        let path = path.to_string();
        let path = Path::new(&path);
        let host_build_results =
            match Host::try_new_from_clap_file(path, sample_rate, max_latency_seconds) {
                Ok(host_build_results) => host_build_results,
                Err(err) => {
                    godot_warn!("{err}");
                    return Array::new();
                }
            };
        host_build_results
            .into_iter()
            .filter_map(|host_build_result| {
                let host = match host_build_result {
                    Ok(host) => host,
                    Err(err) => {
                        godot_warn!("{err}");
                        return None;
                    }
                };

                Some(Self::new(host))
            })
            .collect()
    }

    /// 通过多个文件路径，加载内部包含的所有Clap插件。
    /// - `paths`: 各个clap插件文件系统路径。
    /// - `sample_rate`: 采样率，决定了本处理实例读取和写入音频缓冲的整体帧速率。
    /// - `max_latency_seconds`: 最大延迟秒数。代表着Clap插件处理完音频信号之后，这些输出信号最多会在缓存里面留多长时间。用这个参数乘上采样率就是缓冲区的最大帧长度。[color=yellow]注意：这个值不要过小，否则clap插件线程数据的读取线程遭遇帧率扰动时会容易导致卡顿。如果这个值小于读取线程的最小帧间隔（指`_process`的`delta`），则会无法正常读取音频数据。[/color][color=red]也不要太大，内存会炸的。[/color]
    ///
    /// 返回：加载的各个clap实例。
    #[func]
    fn new_from_clap_files(
        paths: PackedStringArray,
        sample_rate: f64,
        max_latency_seconds: f64,
    ) -> Array<Gd<Self>> {
        paths
            .as_slice()
            .iter()
            .map(|path| {
                Self::new_from_clap_file(path.clone(), sample_rate, max_latency_seconds)
                    .iter_shared()
                    .collect::<Box<[_]>>()
            })
            .flatten()
            .collect()
    }

    /// 插件描述信息
    #[func]
    fn plugin_desc(&self) -> GString {
        self.host.message_processor().plugin_desc().to_godot()
    }

    /// 显示插件GUI。
    #[func]
    fn show_gui(&self) {
        self.host_shared().request_show().unwrap_or_else(|err| {
            self.log(
                LogSeverity::PluginMisbehaving,
                &format!("显示Gui失败：{err}"),
            );
        });
    }

    /// 隐藏插件GUI。
    #[func]
    fn hide_gui(&self) {
        self.host_shared().request_hide().unwrap_or_else(|err| {
            self.log(
                LogSeverity::PluginMisbehaving,
                &format!("隐藏Gui失败：{err}"),
            );
        });
    }

    /// 获取本插件中的所有输入端口的访问句柄。
    #[func]
    fn input_audio_port_accesses(&self) -> Array<Gd<ClapInputAudioPortAccess>> {
        self.host
            .audio_access()
            .input_audio_port_accesses()
            .iter()
            .cloned()
            .collect()
    }

    /// 获取本插件中的所有输出端口的访问句柄。
    #[func]
    fn output_audio_port_accesses(&self) -> Array<Gd<ClapOutputAudioPortAccess>> {
        self.host
            .audio_access()
            .output_audio_port_accesses()
            .iter()
            .cloned()
            .collect()
    }

    /// 向插件发送midi事件。
    #[func]
    fn push_midi(&self, midi: Array<Gd<InputEventMidi>>) {
        let events = midi.iter_shared().map(|midi| midi_to_event(midi)).collect();
        self.host.audio_access().send_input_event_buffers(events);
    }

    /// 收到来自插件的midi事件。
    #[signal]
    fn midi_received(midi: Array<Gd<InputEventMidi>>);

    #[func]
    fn get_clap_transport_event_access(&self) -> Option<Gd<ClapTransportEventAccess>> {
        self.host
            .audio_access()
            .get_clap_transport_event_access()
            .cloned()
    }
    #[func]
    fn set_clap_transport_event_access(
        &mut self,
        clap_transport_event_access: Option<Gd<ClapTransportEventAccess>>,
    ) {
        self.host
            .set_transport_event_access(clap_transport_event_access);
    }

    #[func]
    fn get_state(&mut self) -> PackedByteArray {
        PackedByteArray::from_iter(self.host.get_state())
    }
    #[func]
    fn set_state(&mut self, state: PackedByteArray) {
        self.host.set_state(state.as_slice());
    }
}
#[godot_api]
impl INode for ClapPluginInstance {
    fn on_notification(&mut self, what: NodeNotification) {
        match what {
            NodeNotification::READY => {
                self.init_window();
            }
            NodeNotification::INTERNAL_PROCESS => {
                // 之所以写在这里而不是 process 方法，是因为 on_notification 方法被子类重写之后依然会被正常调用，而 process 等其他方法被重写之后就会被覆盖掉。
                self.host.process();

                // 插件midi事件触发。
                let midi_received = self
                    .host
                    .audio_access()
                    .output_event_buffer_rx()
                    .try_iter()
                    .flatten()
                    .map(event_to_midi)
                    .collect::<Array<_>>();
                if !midi_received.is_empty() {
                    self.signals().midi_received().emit(&midi_received);
                }
            }
            _ => {}
        }
    }
}
