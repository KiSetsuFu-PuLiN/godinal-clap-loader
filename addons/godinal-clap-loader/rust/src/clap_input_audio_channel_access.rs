use godot::{
    classes::{AudioStream, AudioStreamPlayback},
    prelude::*,
};
use itertools::{EitherOrBoth, Itertools};
use std::sync::mpsc::Sender;

/// 对Clap插件输入通道的访问句柄。
#[derive(GodotClass)]
#[class(no_init)]
pub struct ClapInputAudioChannelAccess {
    channel_buffer_tx: Sender<Box<[f32]>>,

    /// 通道接收音频流。当这个属性被设置后，会以所属Clap插件的采样率速度持续不断地从流中读取数据并发往音频线程对应的通道缓冲。
    ///
    /// 注意：通道仅包含一个声道，故而传入的双声道数据会在根据其属性转为单声道数据：
    /// - 若[`AudioStream::is_monophonic()`]为`true`：则只取左声道数据
    /// - 若[`AudioStream::is_monophonic()`]为`false`：则取双声道数据的均值。
    #[var(get=get_stream, set=set_stream)]
    stream: Option<Gd<AudioStream>>,
    stream_playback: Option<Gd<AudioStreamPlayback>>,
}
#[godot_api]
impl ClapInputAudioChannelAccess {
    pub fn new(channel_buffer_tx: Sender<Box<[f32]>>) -> Gd<Self> {
        Gd::from_object(Self {
            channel_buffer_tx,
            stream: None,
            stream_playback: None,
        })
    }

    fn push_buffer(&self, frames: Box<[f32]>) {
        self.channel_buffer_tx.send(frames).unwrap_or_else(|err| {
            godot_error!(
                "ClapInputAudioChannelAccess对应的输入通道已不复存在：{:?}",
                err
            )
        })
    }

    pub fn process(&mut self, port_buffer: impl ExactSizeIterator<Item = f32>) {
        let is_monophonic = self
            .stream
            .as_ref()
            .map(|stream| stream.is_monophonic())
            .unwrap_or(true);

        let buffer = self
            .stream_playback
            .as_mut()
            .map(|stream_playback| stream_playback.mix_audio(1.0, port_buffer.len() as i32));
        let buffer = buffer
            .as_ref()
            .map_or(&[] as &[Vector2], |buffer| buffer.as_slice());

        let buffer = buffer.iter().zip_longest(port_buffer);
        let buffer = if is_monophonic {
            buffer
                .map(|frame| match frame {
                    EitherOrBoth::Both(channel_frame, port_frame) => channel_frame.x + port_frame,
                    EitherOrBoth::Left(..) => {
                        unreachable!(
                            "经过mix_audio限制之后，channel buffer 不应该比 port buffer 长"
                        )
                    }
                    EitherOrBoth::Right(port_frame) => port_frame,
                })
                .collect()
        } else {
            buffer
                .map(|frame| match frame {
                    EitherOrBoth::Both(channel_frame, port_frame) => {
                        (channel_frame.x + channel_frame.y) / 2.0 + port_frame
                    }
                    EitherOrBoth::Left(..) => {
                        unreachable!(
                            "经过mix_audio限制之后，channel buffer 不应该比 port buffer 长"
                        )
                    }
                    EitherOrBoth::Right(port_frame) => port_frame,
                })
                .collect()
        };

        self.push_buffer(buffer);
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
