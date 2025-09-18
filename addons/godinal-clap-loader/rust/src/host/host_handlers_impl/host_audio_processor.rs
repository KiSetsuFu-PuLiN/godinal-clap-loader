use crate::host::host_handlers_impl::{host_main_thread::HostMainThread, host_shared::HostShared};
use clack_host::host::AudioProcessorHandler;

/// 用于为启动音频处理实例提供支持。
/// 也可访问[`HostShared`]和[`HostMainThread`]。
pub struct HostAudioProcessor<'a> {
    host_shared: &'a HostShared,
    host_main_thread: HostMainThread<'a>,
}
impl<'a> HostAudioProcessor<'a> {
    pub fn new(host_shared: &'a HostShared, host_main_thread: HostMainThread<'a>) -> Self {
        Self {
            host_shared,
            host_main_thread,
        }
    }
}
impl<'a> AudioProcessorHandler<'a> for HostAudioProcessor<'a> {}
