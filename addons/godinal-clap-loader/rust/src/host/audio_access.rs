use crate::{
    clap_input_audio_port_access::ClapInputAudioPortAccess,
    clap_output_audio_port_access::ClapOutputAudioPortAccess,
    clap_transport_event_access::ClapTransportEventAccess,
};
use clack_host::events::{UnknownEvent, event_types::TransportEvent};
use godot::{prelude::*, register::ConnectHandle};
use std::{
    mem::swap,
    sync::{
        Arc, RwLock,
        mpsc::{Receiver, Sender},
    },
    time::{Duration, SystemTime},
};

/// 插件音频线程的访问句柄。
pub struct AudioAccess {
    process_time: SystemTime,

    input_audio_port_accesses: Box<[Gd<ClapInputAudioPortAccess>]>,
    output_audio_port_accesses: Box<[Gd<ClapOutputAudioPortAccess>]>,

    input_event_buffer_tx: Sender<Box<[Box<UnknownEvent>]>>,
    output_event_buffer_rx: Receiver<Box<[Box<UnknownEvent>]>>,

    transport_event: Arc<RwLock<Option<TransportEvent>>>,
    clap_transport_event_access: Option<(Gd<ClapTransportEventAccess>, ConnectHandle)>,
}
impl AudioAccess {
    pub fn new(
        input_audio_port_accesses: Box<[Gd<ClapInputAudioPortAccess>]>,
        output_audio_port_accesses: Box<[Gd<ClapOutputAudioPortAccess>]>,
        input_event_buffer_tx: Sender<Box<[Box<UnknownEvent>]>>,
        output_event_buffer_rx: Receiver<Box<[Box<UnknownEvent>]>>,
        transport_event: Arc<RwLock<Option<TransportEvent>>>,
    ) -> Self {
        Self {
            process_time: SystemTime::now(),
            input_audio_port_accesses,
            output_audio_port_accesses,
            input_event_buffer_tx,
            output_event_buffer_rx,
            transport_event,
            clap_transport_event_access: None,
        }
    }

    pub fn input_audio_port_accesses(&self) -> &[Gd<ClapInputAudioPortAccess>] {
        &self.input_audio_port_accesses
    }
    pub fn output_audio_port_accesses(&self) -> &[Gd<ClapOutputAudioPortAccess>] {
        &self.output_audio_port_accesses
    }

    pub fn send_input_event_buffers(&self, input_event_buffers: Box<[Box<UnknownEvent>]>) {
        self.input_event_buffer_tx
            .send(input_event_buffers)
            .unwrap_or_else(|err| {
                godot_error!("ClapPluginInstance对应的事件输入缓冲已不复存在：{:?}", err)
            });
    }
    pub fn output_event_buffer_rx(&self) -> &Receiver<Box<[Box<UnknownEvent>]>> {
        &self.output_event_buffer_rx
    }

    pub fn get_clap_transport_event_access(&self) -> Option<&Gd<ClapTransportEventAccess>> {
        self.clap_transport_event_access
            .as_ref()
            .map(|(clap_transport_event_access, _)| clap_transport_event_access)
    }
    pub fn set_clap_transport_event_access(
        &mut self,
        clap_transport_event_access: Option<Gd<ClapTransportEventAccess>>,
    ) {
        let mut clap_transport_event_access = if let Some(clap_transport_event_access) =
            clap_transport_event_access
        {
            let transport_event = self.transport_event.clone();
            let connection_handle = clap_transport_event_access
                .signals()
                .value_changed()
                .connect_self(move |this| {
                    let mut transport_event = transport_event.write().unwrap_or_else(|err|panic!("向音频线程设置transport_event状态时出错，写锁获取失败，有可能是其他句柄在设置这个属性的时候出了什么意外：{err}"));
                    transport_event.replace(*this.transport_event());
                });
            Some((clap_transport_event_access, connection_handle))
        } else {
            None
        };

        swap(
            &mut clap_transport_event_access,
            &mut self.clap_transport_event_access,
        );
        if let Some((_, connection_handle)) = clap_transport_event_access {
            connection_handle.disconnect();
        }
    }

    fn process_input_audio(&mut self, time: f64) {
        for input_audio_port_access in &mut self.input_audio_port_accesses {
            input_audio_port_access.bind_mut().process(time);
        }
    }

    fn process_output_audio(&mut self) {
        for output_audio_port_access in &mut self.output_audio_port_accesses {
            output_audio_port_access.bind_mut().process();
        }
    }

    pub fn process(&mut self) {
        let time = SystemTime::now()
            .duration_since(self.process_time)
            .unwrap_or_else(|system_time_error| {
                eprintln!("插件访问句柄已运行时间计算错误：{system_time_error}");
                Duration::ZERO
            });
        self.process_input_audio(time.as_secs_f64());
        self.process_output_audio();
    }

    // todo: 添加cv控制。
}
