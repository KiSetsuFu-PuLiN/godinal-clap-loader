use crate::host::host_handlers_impl::host_shared::HostShared;
use clack_extensions::{
    params::{HostParamsImplMainThread, ParamClearFlags, ParamRescanFlags},
    state::HostStateImpl,
    timer::{HostTimerImpl, TimerId},
};
use clack_host::{
    extensions::{Extension, PluginExtensionSide},
    host::{HostError, MainThreadHandler},
    plugin::InitializedPluginHandle,
    utils::ClapId,
};
use std::cell::OnceCell;

/// 插件总句柄，用于访问插件各种功能的句柄。
/// 也可访问[`HostShared`]。
#[derive(Clone)]
pub struct HostMainThread<'a> {
    /// 共享宿主数据的引用。
    host_shared: &'a HostShared,
    /// 插件实例的句柄。
    plugin_handle: OnceCell<InitializedPluginHandle<'a>>,
}
impl<'a> HostMainThread<'a> {
    pub fn new(host_shared: &'a HostShared) -> Self {
        Self {
            host_shared,
            plugin_handle: OnceCell::new(),
        }
    }

    /// 返回插件侧的各种功能的句柄。
    pub fn get_extension<E: Extension<ExtensionSide = PluginExtensionSide>>(&self) -> Option<E> {
        self.plugin_handle
            .get()
            .map(|plugin_handle| plugin_handle.get_extension())
            .flatten()
    }
}
impl<'a> MainThreadHandler<'a> for HostMainThread<'a> {
    fn initialized(&mut self, instance: InitializedPluginHandle<'a>) {
        self.plugin_handle
            .set(instance)
            .unwrap_or_else(|err| panic!("HostMainThread疑似被初始化了多次：{:?}", err));
    }
}
impl<'a> HostTimerImpl for HostMainThread<'a> {
    fn register_timer(&mut self, period_ms: u32) -> Result<TimerId, HostError> {
        todo!()
    }

    fn unregister_timer(&mut self, timer_id: TimerId) -> Result<(), HostError> {
        todo!()
    }
}
impl<'a> HostParamsImplMainThread for HostMainThread<'a> {
    fn rescan(&mut self, flags: ParamRescanFlags) {
        for flag in flags.iter() {
            match flag {
                ParamRescanFlags::VALUES
                | ParamRescanFlags::INFO
                | ParamRescanFlags::TEXT
                | ParamRescanFlags::ALL => {
                    //todo: 大概需要通知外层宿主重新扫描所有的Param参数控制。
                }
                flag => unreachable!("未处理的标志：{:?}", flag),
            }
        }
    }

    fn clear(&mut self, param_id: ClapId, flags: ParamClearFlags) {
        todo!()
    }
}
impl<'a> HostStateImpl for HostMainThread<'a> {
    fn mark_dirty(&mut self) {
        todo!()
    }
}
