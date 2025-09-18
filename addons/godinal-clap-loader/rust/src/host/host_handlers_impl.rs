pub mod host_audio_processor;
pub mod host_main_thread;
pub mod host_shared;

use crate::host::{
    Host,
    host_handlers_impl::{
        host_audio_processor::HostAudioProcessor, host_main_thread::HostMainThread,
        host_shared::HostShared,
    },
};
use clack_extensions::{
    gui::HostGui, log::HostLog, params::HostParams, state::HostState, timer::HostTimer,
};
use clack_host::host::{HostExtensions, HostHandlers};

impl HostHandlers for Host {
    type Shared<'a> = HostShared;
    type MainThread<'a> = HostMainThread<'a>;
    type AudioProcessor<'a> = HostAudioProcessor<'a>;

    /// 声明本主机侧实现了哪些句柄可供插件使用。
    /// 声明之后均需要在 [`HostShared`] 或 [`HostMainThread`] 中实现。
    #[allow(unused)]
    fn declare_extensions(builder: &mut HostExtensions<Self>, shared: &Self::Shared<'_>) {
        builder
            .register::<HostLog>()
            .register::<HostGui>()
            .register::<HostTimer>()
            .register::<HostParams>()
            .register::<HostState>();
    }
}
