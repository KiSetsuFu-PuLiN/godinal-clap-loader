use crate::{
    clap_input_audio_channel_access::ClapInputAudioChannelAccess,
    clap_output_audio_channel_access::ClapOutputAudioChannelAccess,
};
use godot::{classes::AudioStreamGenerator, prelude::*};
use std::{
    collections::VecDeque,
    mem::swap,
    sync::mpsc::{Receiver, Sender, channel},
};

// todo: 缓冲区使用一整块的内存或许性能会更好。

pub struct InputChannel<F> {
    /// 接受Godot输入的通道。
    channel_buffer_rx: Receiver<Box<[F]>>,
    /// 缓冲区的帧大小，这个长度限制用于同步延迟，防止本应在卡顿下丢失的帧数据长期滞留在缓存中。
    frames_count: usize,
    /// 来自Godot，当前已接收并滞留的音频数据。
    buffer: VecDeque<F>,
    /// 已经送往插件的音频数据。
    active_buffer: Vec<F>,
}
impl<F> InputChannel<F> {
    pub fn process(&mut self) {
        let space_left = self.frames_count - self.buffer.len();
        self.buffer
            .extend(self.channel_buffer_rx.try_iter().flatten().take(space_left))
    }
}
impl<F: Clone> InputChannel<F> {
    pub fn pop_buffer(
        &mut self,
        frames_count: usize,
        empty_value: F,
    ) -> clack_host::prelude::InputChannel<'_, F> {
        let is_constant = self.buffer.is_empty();
        let drain_count = frames_count.min(self.buffer.len());
        self.active_buffer.clear();
        self.active_buffer.extend(self.buffer.drain(..drain_count));
        // 需要务必保持长度和输出缓冲的一致，否则在没有输入时会放不出声音。
        self.active_buffer.resize(frames_count, empty_value);
        clack_host::prelude::InputChannel::from_buffer(&mut self.active_buffer, is_constant)
    }
}
impl InputChannel<f32> {
    pub fn new(frames_count: usize) -> (Self, Gd<ClapInputAudioChannelAccess>) {
        let (channel_buffer_tx, channel_buffer_rx) = channel();
        (
            Self {
                channel_buffer_rx,
                frames_count,
                buffer: VecDeque::with_capacity(frames_count),
                active_buffer: Vec::with_capacity(frames_count),
            },
            ClapInputAudioChannelAccess::new(channel_buffer_tx),
        )
    }
}

pub struct OutputChannel<F> {
    /// 来自Clap插件，当前已接受并滞留的音频数据。
    buffer: Vec<F>,
    /// 数据发往Godot的通道。
    channel_buffer_tx: Sender<Box<[F]>>,
}
impl<F> OutputChannel<F> {
    pub fn process(&mut self) {
        let mut buffer = Vec::new();
        swap(&mut buffer, &mut self.buffer);
        self.channel_buffer_tx
            .send(buffer.into())
            .unwrap_or_else(|err| {
                panic!(
                    "音频缓冲输出通道寄了，ClapPluginInstance大概已经被销毁，本缓冲所在的线程应该也会很快销毁：{:?}",
                    err
                )
            })
    }
}
impl<F: Clone> OutputChannel<F> {
    pub fn pop_buffer(&mut self, frames_count: usize, empty_value: F) -> &mut [F] {
        let start_index = self.buffer.len();
        self.buffer.resize(start_index + frames_count, empty_value);
        &mut self.buffer[start_index..]
    }
}
impl OutputChannel<f32> {
    pub fn new(
        audio_stream_generator: Gd<AudioStreamGenerator>,
    ) -> (Self, Gd<ClapOutputAudioChannelAccess>) {
        let (channel_buffer_tx, channel_buffer_rx) = channel();
        (
            Self {
                buffer: Vec::new(),
                channel_buffer_tx,
            },
            ClapOutputAudioChannelAccess::new(channel_buffer_rx, audio_stream_generator),
        )
    }
}
