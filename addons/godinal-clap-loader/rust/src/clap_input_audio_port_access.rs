use std::iter::repeat_n;

use crate::clap_input_audio_channel_access::ClapInputAudioChannelAccess;
use godot::{
    classes::{AudioStream, AudioStreamPlayback},
    prelude::*,
};

/// 对Clap插件输出端口的访问句柄。
#[derive(GodotClass)]
#[class(no_init)]
pub struct ClapInputAudioPortAccess {
    sample_rate: f64,
    processed_frames_count: usize,
    channel_accesses: Box<[Gd<ClapInputAudioChannelAccess>]>,

    /// 端口接收音频流。当这个属性被设置后，会以所属Clap插件的采样率速度持续不断地从流中读取数据并发往音频线程对应的端口缓冲。
    ///
    /// 注意：传入数据只包含两个声道，故而如果本端口包含两个以上的通道，多出来的通道则不会收到音频数据。
    #[var(get=get_stream, set=set_stream)]
    stream: Option<Gd<AudioStream>>,
    stream_playback: Option<Gd<AudioStreamPlayback>>,
}
#[godot_api]
impl ClapInputAudioPortAccess {
    pub fn new(
        sample_rate: f64,
        channel_accesses: Box<[Gd<ClapInputAudioChannelAccess>]>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            sample_rate,
            processed_frames_count: 0,
            channel_accesses,
            stream: None,
            stream_playback: None,
        })
    }

    pub fn process(&mut self, time: f64) {
        let buffer_frames_count = (time * self.sample_rate) - self.processed_frames_count as f64;
        if buffer_frames_count < 1.0 {
            return;
        }

        let buffer_frames_count = buffer_frames_count as usize;
        let buffer = self
            .stream_playback
            .as_mut()
            .map(|stream_playback| stream_playback.mix_audio(1.0, buffer_frames_count as i32));
        let buffer = buffer
            .as_ref()
            .map_or(&[] as &[Vector2], |buffer| buffer.as_slice());

        let mut stream_buffer = if buffer.is_empty() {
            [None, None]
        } else {
            let (left_frames, right_frames): (Vec<_>, Vec<_>) =
                buffer.iter().map(|value| (value.x, value.y)).unzip();
            [Some(left_frames), Some(right_frames)]
        }
        .into_iter()
        .flatten();

        let buffer = repeat_n(0.0, buffer_frames_count);
        for channel_access in self.channel_accesses.iter_mut() {
            let mut channel_access_bind_mut = channel_access.bind_mut();
            if let Some(mut stream_buffer) = stream_buffer.next() {
                stream_buffer.resize(buffer_frames_count, 0.0);
                channel_access_bind_mut.process(stream_buffer.into_iter());
            } else {
                channel_access_bind_mut.process(buffer.clone());
            }
        }

        self.processed_frames_count += buffer_frames_count;
    }

    /// 获取本端口中的所有通道访问句柄。
    #[func]
    fn channel_accesses(&self) -> Array<Gd<ClapInputAudioChannelAccess>> {
        self.channel_accesses.iter().cloned().collect()
    }

    #[func]
    fn get_stream(&self) -> Option<Gd<AudioStream>> {
        self.stream.clone()
    }
    #[func]
    fn set_stream(&mut self, mut stream: Option<Gd<AudioStream>>) {
        self.stream_playback = stream.as_mut().and_then(|stream| {
            stream.instantiate_playback().map(|mut stream_playback| {
                stream_playback.start();
                stream_playback
            })
        });
        self.stream = stream;
    }
}
