mod audio_buffer;
mod event_buffer;

use crate::host::{
    Host, HostBuildError,
    audio_access::AudioAccess,
    audio_processor::{
        audio_buffer::{
            AudioBuffer,
            channel::{InputChannel, OutputChannel},
        },
        event_buffer::{EventBuffer, InputHandle, OutputHandle},
    },
    host_handlers_impl::host_audio_processor::HostAudioProcessor,
};
use clack_extensions::audio_ports::PluginAudioPorts;
use clack_host::{
    events::event_types::TransportEvent,
    host::HostHandlers,
    plugin::PluginInstance,
    process::{PluginAudioConfiguration, StartedPluginAudioProcessor},
};
use godot::{
    classes::{AudioStreamGenerator, audio_stream_generator::AudioStreamGeneratorMixRate},
    obj::NewGd,
};
use itertools::Itertools;
use std::{
    cell::OnceCell,
    iter::repeat_n,
    sync::{Arc, RwLock, mpsc::channel},
    time::SystemTime,
};

pub struct AudioProcessor<T: HostHandlers, F> {
    /// 插件音频处理器。
    plugin_audio_processor: StartedPluginAudioProcessor<T>,
    /// 音频设置。
    plugin_audio_configuration: PluginAudioConfiguration,

    /// 输入音频缓冲区。
    input_audio_buffer: AudioBuffer<InputChannel<F>>,
    /// 输出音频缓冲区。
    output_audio_buffer: AudioBuffer<OutputChannel<F>>,

    /// 输入事件缓冲区。
    input_event_buffer: EventBuffer<InputHandle>,
    /// 输出事件缓冲区。
    output_event_buffer: EventBuffer<OutputHandle>,

    /// 表示宿主数字音频工作站(DAW)的播放传输状态信息，
    /// 用于同步插件处理与宿主播放时间轴。
    transport_event: Arc<RwLock<Option<TransportEvent>>>,

    /// 本插件实例生成的时间。
    start_time: SystemTime,

    /// 已处理帧数。
    steady_time: u64,
}
impl AudioProcessor<Host, f32> {
    pub fn try_new(
        plugin_instance: &mut PluginInstance<Host>,
        sample_rate: f64,
        max_latency_seconds: f64,
    ) -> Result<(Self, AudioAccess), HostBuildError> {
        let buffer_frames_count = sample_rate * max_latency_seconds;
        if buffer_frames_count < 1.0 {
            return Err(HostBuildError::MaxLatencyMustBeGreaterThanZero);
        }
        let buffer_frames_count = buffer_frames_count as usize;
        let plugin_audio_configuration = PluginAudioConfiguration {
            sample_rate,
            min_frames_count: 1,
            max_frames_count: buffer_frames_count as u32,
        };

        let plugin_audio_processor = plugin_instance
            .activate(
                |host_shared, host_main_thread| {
                    HostAudioProcessor::new(host_shared, host_main_thread.clone())
                },
                plugin_audio_configuration,
            )?
            .start_processing()?;

        let host_shared = plugin_instance.access_shared_handler(|host_shared| host_shared.clone());
        let mut plugin_main_thread_handle = plugin_instance.plugin_handle();
        let plugin_audio_ports = plugin_main_thread_handle
            .get_extension::<PluginAudioPorts>()
            .ok_or(HostBuildError::NoPluginAudioPorts)?;

        let mut audio_stream_generator = AudioStreamGenerator::new_gd();
        audio_stream_generator.set_buffer_length(max_latency_seconds as f32);
        audio_stream_generator.set_mix_rate(sample_rate as f32);
        audio_stream_generator.set_mix_rate_mode(AudioStreamGeneratorMixRate::CUSTOM);

        let (input_audio_buffer, input_audio_port_accesses) = AudioBuffer::<InputChannel<_>>::new(
            &host_shared,
            &mut plugin_main_thread_handle,
            &plugin_audio_ports,
            buffer_frames_count,
            sample_rate,
        );
        let (output_audio_buffer, output_audio_port_accesses) =
            AudioBuffer::<OutputChannel<_>>::new(
                &host_shared,
                &mut plugin_main_thread_handle,
                &plugin_audio_ports,
                audio_stream_generator,
            );

        let (input_event_buffer_tx, input_event_buffer_rx): (_, InputHandle) = channel();
        let input_event_buffer = EventBuffer::<InputHandle>::new(input_event_buffer_rx);
        let (output_event_buffer_tx, output_event_buffer_rx): (OutputHandle, _) = channel();
        let output_event_buffer = EventBuffer::<OutputHandle>::new(output_event_buffer_tx);

        let start_time = SystemTime::now();

        let transport_event = Arc::new(RwLock::new(None));

        Ok((
            Self {
                plugin_audio_processor,
                plugin_audio_configuration,
                input_audio_buffer,
                output_audio_buffer,
                input_event_buffer,
                output_event_buffer,
                transport_event: transport_event.clone(),
                start_time,
                steady_time: 0,
            },
            AudioAccess::new(
                input_audio_port_accesses,
                output_audio_port_accesses,
                input_event_buffer_tx,
                output_event_buffer_rx,
                transport_event,
            ),
        ))
    }

    fn process_batches_before(&mut self) {
        self.input_audio_buffer.process();
        self.input_event_buffer.process();
    }

    fn process_batch(&mut self, buffer_frames_count: usize, transport: Option<&TransportEvent>) {
        let input_audio_buffer = self.input_audio_buffer.pop_buffer(buffer_frames_count);
        let mut output_audio_buffer = self.output_audio_buffer.pop_buffer(buffer_frames_count);

        let input_events = self.input_event_buffer.pop_buffer();
        let mut output_events = self.output_event_buffer.pop_buffer();

        match self.plugin_audio_processor.process(
            &input_audio_buffer,
            &mut output_audio_buffer,
            &input_events,
            &mut output_events,
            Some(self.steady_time),
            transport,
        ) {
            Ok(process_status) => {
                // todo: 对process_status进行处理，优化本process方法的执行时机。
            }
            Err(plugin_instance_error) => {
                eprintln!("音频处理运行时错误：{plugin_instance_error}");
            }
        }

        self.steady_time += buffer_frames_count as u64;
    }

    fn process_batches_after(&mut self) {
        self.output_audio_buffer.process();
        self.output_event_buffer.process();
    }

    pub fn process(&mut self) {
        let transport_event = self.transport_event.clone();
        let transport_event = transport_event.read().unwrap_or_else(|err| {
            panic!(
                "获取transport_event的读权限失败，大概是宿主在更改这个属性的时候出了什么意外：{err}"
            )
        });
        let transport_event = transport_event.as_ref();

        let time = match SystemTime::now().duration_since(self.start_time) {
            Ok(time_delta) => time_delta,
            Err(system_time_error) => {
                eprintln!("插件已运行时间计算错误：{system_time_error}");
                return;
            }
        };
        let frames_count_delta = (time.as_secs_f64() * self.plugin_audio_configuration.sample_rate)
            - self.steady_time as f64;
        if frames_count_delta < 1.0 {
            return;
        }

        let frames_count_delta = frames_count_delta as usize;
        let frames_counts = repeat_n((), frames_count_delta)
            .chunks(self.plugin_audio_configuration.max_frames_count as usize);
        let frames_counts = frames_counts
            .into_iter()
            .map(|frames_count| frames_count.count());

        self.process_batches_before();
        let is_processed = OnceCell::<()>::new();
        for frames_count in frames_counts {
            self.process_batch(frames_count, transport_event);
            is_processed.get_or_init(|| ());
        }
        if is_processed.get().is_some() {
            self.process_batches_after();
        }
    }
}
