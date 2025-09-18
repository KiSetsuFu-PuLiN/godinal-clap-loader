pub mod channel;
mod port;

use crate::{
    clap_input_audio_port_access::ClapInputAudioPortAccess,
    clap_output_audio_port_access::ClapOutputAudioPortAccess,
    host::{
        audio_processor::audio_buffer::{
            channel::{InputChannel, OutputChannel},
            port::Port,
        },
        host_handlers_impl::host_shared::HostShared,
    },
};
use clack_extensions::{
    audio_ports::{AudioPortInfo, AudioPortInfoBuffer, PluginAudioPorts},
    log::{HostLogImpl, LogSeverity},
};
use clack_host::{
    plugin::PluginMainThreadHandle,
    prelude::{AudioPorts, InputAudioBuffers, OutputAudioBuffers},
};
use godot::{classes::AudioStreamGenerator, obj::Gd};
use itertools::multiunzip;

pub struct AudioBuffer<Channel> {
    audio_prots: AudioPorts,
    ports: Box<[Port<Channel>]>,
}

impl<F: Clone> AudioBuffer<InputChannel<F>> {
    pub fn process(&mut self) {
        for audio_port_buffer in &mut self.ports {
            audio_port_buffer.process();
        }
    }
}
impl AudioBuffer<InputChannel<f32>> {
    pub fn new(
        host_shared: &HostShared,
        plugin_main_thread_handle: &mut PluginMainThreadHandle,
        plugin_audio_ports: &PluginAudioPorts,
        frames_count: usize,
        sample_rate: f64,
    ) -> (Self, Box<[Gd<ClapInputAudioPortAccess>]>) {
        let mut buffer = AudioPortInfoBuffer::new();

        let audio_port_buffers_count =
            plugin_audio_ports.count(plugin_main_thread_handle, true) as usize;

        let (ports, clap_input_audio_port_accesses, channel_counts): (Vec<_>, Vec<_>, Vec<_>) =
            multiunzip((0..audio_port_buffers_count).filter_map(|index| {
                let Some(audio_port_info) = assert_get_audio_port_info(
                    plugin_audio_ports,
                    plugin_main_thread_handle,
                    index as u32,
                    true,
                    &mut buffer,
                    host_shared,
                ) else {
                    return None;
                };

                let (port, clap_input_audio_channel_access) = Port::<InputChannel<_>>::new(
                    audio_port_info.channel_count as usize,
                    frames_count,
                    sample_rate,
                );

                Some((
                    port,
                    clap_input_audio_channel_access,
                    audio_port_info.channel_count as usize,
                ))
            }));

        (
            Self {
                audio_prots: audio_prots(&channel_counts),
                ports: ports.into(),
            },
            clap_input_audio_port_accesses.into(),
        )
    }

    pub fn pop_buffer(&mut self, frames_count: usize) -> InputAudioBuffers<'_> {
        self.audio_prots.with_input_buffers(
            self.ports
                .iter_mut()
                .map(|port| port.pop_buffer(frames_count)),
        )
    }
}
#[deprecated(note = "在以Cardinal.clap进行开发测试时，发现f64将导致段错误。故不要使用这个实现。")]
#[allow(deprecated, unused)]
impl AudioBuffer<InputChannel<f64>> {
    pub fn pop_buffer(&mut self, frames_count: usize) -> InputAudioBuffers<'_> {
        self.audio_prots.with_input_buffers(
            self.ports
                .iter_mut()
                .map(|port| port.pop_buffer(frames_count)),
        )
    }
}

impl<F: Clone> AudioBuffer<OutputChannel<F>> {
    pub fn process(&mut self) {
        for audio_port_buffer in &mut self.ports {
            audio_port_buffer.process();
        }
    }
}
impl AudioBuffer<OutputChannel<f32>> {
    pub fn new(
        host_shared: &HostShared,
        plugin_main_thread_handle: &mut PluginMainThreadHandle,
        plugin_audio_ports: &PluginAudioPorts,
        audio_stream_generator: Gd<AudioStreamGenerator>,
    ) -> (Self, Box<[Gd<ClapOutputAudioPortAccess>]>) {
        let mut buffer = AudioPortInfoBuffer::new();

        let audio_port_buffers_count =
            plugin_audio_ports.count(plugin_main_thread_handle, false) as usize;
        let (ports, clap_output_audio_port_accesses, channel_counts): (Vec<_>, Vec<_>, Vec<_>) =
            multiunzip((0..audio_port_buffers_count).filter_map(|index| {
                let Some(audio_port_info) = assert_get_audio_port_info(
                    plugin_audio_ports,
                    plugin_main_thread_handle,
                    index as u32,
                    false,
                    &mut buffer,
                    host_shared,
                ) else {
                    return None;
                };

                let (port, clap_output_audio_channel_access) = Port::<OutputChannel<_>>::new(
                    audio_port_info.channel_count as usize,
                    audio_stream_generator.clone(),
                );

                Some((
                    port,
                    clap_output_audio_channel_access,
                    audio_port_info.channel_count as usize,
                ))
            }));

        (
            Self {
                audio_prots: audio_prots(&channel_counts),
                ports: ports.into(),
            },
            clap_output_audio_port_accesses.into(),
        )
    }

    pub fn pop_buffer(&mut self, frames_count: usize) -> OutputAudioBuffers<'_> {
        self.audio_prots.with_output_buffers(
            self.ports
                .iter_mut()
                .map(|port| port.pop_buffer(frames_count)),
        )
    }
}
#[deprecated(note = "在以Cardinal.clap进行开发测试时，发现f64将导致段错误。故不要使用这个实现。")]
#[allow(deprecated, unused)]
impl AudioBuffer<OutputChannel<f64>> {
    pub fn pop_buffer(&mut self, frames_count: usize) -> OutputAudioBuffers<'_> {
        self.audio_prots.with_output_buffers(
            self.ports
                .iter_mut()
                .map(|port| port.pop_buffer(frames_count)),
        )
    }
}

fn assert_get_audio_port_info<'a>(
    plugin_audio_ports: &PluginAudioPorts,
    plugin_main_thread_handle: &mut PluginMainThreadHandle<'_>,
    index: u32,
    is_input: bool,
    buffer: &'a mut AudioPortInfoBuffer,
    host_shared: &HostShared,
) -> Option<AudioPortInfo<'a>> {
    let Some(audio_port_info) =
        plugin_audio_ports.get(plugin_main_thread_handle, index, is_input, buffer)
    else {
        host_shared.log(
            LogSeverity::PluginMisbehaving,
            &format!(
                "获取编号为{index}的插件端口（{}）失败",
                if is_input { "输入" } else { "输出" }
            ),
        );
        return None;
    };

    if audio_port_info.channel_count <= 0 {
        host_shared.log(
            LogSeverity::PluginMisbehaving,
            &format!(
                "获取编号为{index}的插件端口（{}）不合要求：通道数量不为正数。",
                if is_input { "输入" } else { "输出" }
            ),
        );
        return None;
    }

    Some(audio_port_info)
}

fn audio_prots(channel_counts: &[usize]) -> AudioPorts {
    AudioPorts::with_capacity(channel_counts.iter().sum(), channel_counts.len())
}
