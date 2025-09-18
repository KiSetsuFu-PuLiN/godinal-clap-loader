mod audio_access;
mod audio_processor;
pub mod host_handlers_impl;
mod message_processor;
mod plugin_message;

use crate::{
    clap_transport_event_access::ClapTransportEventAccess,
    host::{
        audio_access::AudioAccess,
        audio_processor::AudioProcessor,
        host_handlers_impl::{host_main_thread::HostMainThread, host_shared::HostShared},
        message_processor::{MessageProcessor, message_processor_impl::MessageProcessorImpl},
        plugin_message::PluginMessage,
    },
};
use clack_extensions::state::PluginState;
use clack_host::{
    bundle::{PluginBundle, PluginBundleError},
    factory::PluginDescriptor,
    host::{HostHandlers, HostInfo},
    plugin::{PluginInstance, PluginInstanceError},
    process::ProcessingStartError,
};
use godot::prelude::*;
use std::{
    error::Error,
    fmt::Display,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{
        LazyLock,
        mpsc::{Receiver, channel},
    },
    thread::{JoinHandle, spawn},
};

/// 主机数据结构，一个主机承载一个插件
pub struct Host {
    /// [`插件消息`](PluginMessage)处理器
    message_processor: MessageProcessor<Self>,

    /// 音频线程
    audio_processor_thread: Option<JoinHandle<()>>,
    /// 音频线程访问句柄
    audio_access: AudioAccess,
}
impl Host {
    fn try_new(
        plugin_rx: Receiver<PluginMessage>,
        mut plugin_instance: PluginInstance<Self>,
        sample_rate: f64,
        max_latency_seconds: f64,
    ) -> Result<Self, HostBuildError> {
        let (mut audio_processor, audio_access) =
            AudioProcessor::try_new(&mut plugin_instance, sample_rate, max_latency_seconds)?;
        let message_processor = MessageProcessor::<Host>::new(plugin_rx, plugin_instance);

        let audio_processor_thread = Some(spawn(move || {
            loop {
                audio_processor.process();
            }
        }));

        Ok(Self {
            message_processor,
            audio_processor_thread,
            audio_access,
        })
    }
    fn try_new_from_plugin_descriptor(
        plugin_bundle: &PluginBundle,
        plugin_descriptor: PluginDescriptor,
        sample_rate: f64,
        max_latency_seconds: f64,
    ) -> Result<Self, HostBuildError> {
        let plugin_id =
            plugin_descriptor
                .id()
                .ok_or(HostBuildError::InvalidePluginDescriptorMemberValue {
                    member_name: "id".to_string(),
                })?;
        let (plugin_tx, plugin_rx) = channel();
        let plugin_instance = PluginInstance::<Self>::new(
            |()| HostShared::new(plugin_tx),
            |host_shared| HostMainThread::new(host_shared),
            plugin_bundle,
            plugin_id,
            HOST_INFO
                .as_ref()
                .ok_or(HostBuildError::HostInfoBuildFailed)?,
        )?;
        Self::try_new(plugin_rx, plugin_instance, sample_rate, max_latency_seconds)
    }
    pub fn try_new_from_clap_file(
        path: &Path,
        sample_rate: f64,
        max_latency_seconds: f64,
    ) -> Result<Box<[Result<Self, HostBuildError>]>, ClapFileError> {
        let plugin_bundle = unsafe { PluginBundle::load(path) }.map_err(|err| {
            ClapFileError::PluginBundleError {
                path: path.to_path_buf(),
                plugin_bundle_error: err,
            }
        })?;
        let plugin_fatory =
            plugin_bundle
                .get_plugin_factory()
                .ok_or(ClapFileError::NoPluginFactory {
                    path: path.to_path_buf(),
                })?;
        let hosts = plugin_fatory
            .into_iter()
            .map(|plugin_descriptor| {
                Self::try_new_from_plugin_descriptor(
                    &plugin_bundle,
                    plugin_descriptor,
                    sample_rate,
                    max_latency_seconds,
                )
            })
            .collect();
        Ok(hosts)
    }

    /// 插件实例，兼插件消息处理器
    pub fn message_processor(&self) -> &dyn MessageProcessorImpl<Self> {
        self.message_processor.deref()
    }

    /// 获取插件的持久化状态
    pub fn get_state(&mut self) -> Box<[u8]> {
        let mut plugin_main_thread_handle =
            self.message_processor.plugin_instance_mut().plugin_handle();
        let Some(plugin_state) = plugin_main_thread_handle.get_extension::<PluginState>() else {
            godot_error!("获取插件的持久化状态失败，当前Clap插件不支持这个功能");
            return Box::new([]);
        };

        let mut state = Vec::new();
        plugin_state
            .save(&mut plugin_main_thread_handle, &mut state)
            .unwrap_or_else(|err| {
                godot_error!("获取插件持久化状态失败：{err}");
                state.clear();
            });
        state.into_boxed_slice()
    }

    /// 设置插件的状态，让插件从持久化数据恢复
    pub fn set_state(&mut self, mut state: &[u8]) {
        let mut plugin_main_thread_handle =
            self.message_processor.plugin_instance_mut().plugin_handle();
        let Some(plugin_state) = plugin_main_thread_handle.get_extension::<PluginState>() else {
            godot_error!("设置插件的持久化状态失败，当前Clap插件不支持这个功能");
            return;
        };

        plugin_state
            .load(&mut plugin_main_thread_handle, &mut state)
            .unwrap_or_else(|err| {
                godot_error!("设置插件的持久化状态失败：{err}");
            });
    }

    /// 输出音频端口访问句柄。
    pub fn audio_access(&self) -> &AudioAccess {
        &self.audio_access
    }

    /// 设置宿主状态。
    pub fn set_transport_event_access(
        &mut self,
        transport_event_access: Option<Gd<ClapTransportEventAccess>>,
    ) {
        self.audio_access
            .set_clap_transport_event_access(transport_event_access);
    }

    /// 主循环，需要一直调用。
    pub fn process(&mut self) {
        self.message_processor.process();
        self.audio_processor_thread
            .take_if(|audio_processor_thread| {
                if !audio_processor_thread.is_finished() {
                    self.audio_access.process();
                    return false;
                }
                true
            })
            .map(|audio_processor_thread| {
                // 在线程中调用[`HostShared`]的`log`方法，
                // 或者调用godot_print、godot_warn、godot_err等方法，会导致线程终止。
                godot_error!(
                    "音频线程好像有一点似了：{:?}",
                    audio_processor_thread.join().map_err(|err| {
                        if let Some(err) = err.downcast_ref::<&'static str>() {
                            err.to_string()
                        } else if let Some(err) = err.downcast_ref::<String>() {
                            err.clone()
                        } else {
                            "未知错误".to_string()
                        }
                    })
                );
            });
    }
}

// 主机信息，惰性初始化
static HOST_INFO: LazyLock<Option<HostInfo>> = LazyLock::new(|| {
    HostInfo::new(
        "Godinal Clack CPAL Host",
        "KiSetsufu PuLiN",
        "https://space.bilibili.com/37542591",
        env!("CARGO_PKG_VERSION"),
    )
    .ok()
});

/// 创建Host时会出的错
#[derive(Debug)]
pub enum HostBuildError {
    HostInfoBuildFailed,
    InvalidePluginDescriptorMemberValue { member_name: String },
    MaxLatencyMustBeGreaterThanZero,
    PluginInstanceError(PluginInstanceError),
    NoPluginAudioPorts,
}
impl Error for HostBuildError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            HostBuildError::PluginInstanceError(plugin_instance_error) => {
                Some(plugin_instance_error)
            }
            HostBuildError::HostInfoBuildFailed
            | HostBuildError::InvalidePluginDescriptorMemberValue { .. }
            | HostBuildError::MaxLatencyMustBeGreaterThanZero
            | HostBuildError::NoPluginAudioPorts => None,
        }
    }
}
impl Display for HostBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostBuildError::HostInfoBuildFailed => write!(f, "主机信息构建失败，需要检查HOST_INFO"),
            HostBuildError::InvalidePluginDescriptorMemberValue { member_name } => {
                write!(f, "插件描述信息成员不正确，成员名：{member_name}")
            }
            HostBuildError::MaxLatencyMustBeGreaterThanZero => write!(f, "插件的最大延迟必须大于0"),
            HostBuildError::PluginInstanceError(plugin_instance_error) => {
                write!(f, "插件实例错误：{plugin_instance_error}")
            }
            HostBuildError::NoPluginAudioPorts => write!(f, "插件没有找到音频功能支持"),
        }
    }
}
impl From<PluginInstanceError> for HostBuildError {
    fn from(plugin_instance_error: PluginInstanceError) -> Self {
        Self::PluginInstanceError(plugin_instance_error)
    }
}
impl<H: HostHandlers> From<ProcessingStartError<H>> for HostBuildError {
    fn from(processing_start_error: ProcessingStartError<H>) -> Self {
        let plugin_instance_error: PluginInstanceError = processing_start_error.into();
        plugin_instance_error.into()
    }
}

/// 读取Clap文件时会出的错
#[derive(Debug)]
pub enum ClapFileError {
    PluginBundleError {
        path: PathBuf,
        plugin_bundle_error: PluginBundleError,
    },
    NoPluginFactory {
        path: PathBuf,
    },
}
impl Error for ClapFileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ClapFileError::PluginBundleError {
                plugin_bundle_error,
                ..
            } => Some(plugin_bundle_error),
            ClapFileError::NoPluginFactory { .. } => None,
        }
    }
}
impl Display for ClapFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClapFileError::PluginBundleError {
                path,
                plugin_bundle_error,
            } => write!(
                f,
                "Clap文件绑定错误，{}，来自：{}",
                plugin_bundle_error,
                path.display(),
            ),
            ClapFileError::NoPluginFactory { path } => {
                write!(f, "Clap文件不包含插件工厂，来自：{}", path.display())
            }
        }
    }
}
