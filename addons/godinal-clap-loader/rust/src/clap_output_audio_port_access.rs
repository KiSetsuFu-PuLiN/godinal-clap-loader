use crate::{
    clap_output_audio_channel_access::ClapOutputAudioChannelAccess,
    weak_ref::{assert_from_weak_ref, assert_to_weak_ref},
};
use godot::{
    classes::{
        AudioStreamGenerator, AudioStreamGeneratorPlayback, AudioStreamPlayback, IAudioStream,
        WeakRef,
    },
    prelude::*,
};
use std::{cell::OnceCell, ops::Deref};

/// 对Clap插件输出端口的访问句柄。
///
/// 可以被当作[`IAudioStream`]使用，需要注意[`IAudioStream`]仅支持两个声道输出：
/// - 当本Clap输出端口包含少于两个通道时，左右声道的音频数据均来自Clap插件的第一个输出通道。
/// - 当本Clap输出端口包含多余两个通道时，第三个及之后的通道的数据并不会通过[`IAudioStream`]的方式传递出来。如有访问这些通道的需要，请使用[`ClapOutputAudioPortAccess::channel_accesses`]。
#[derive(GodotClass)]
#[class(no_init,base = AudioStream)]
pub struct ClapOutputAudioPortAccess {
    channel_accesses: Box<[Gd<ClapOutputAudioChannelAccess>]>,
    audio_stream_generator: Gd<AudioStreamGenerator>,
    audio_stream_generator_playback_weak_refs: Array<Gd<WeakRef>>,
    /// 由于左右两个通道不一定会同时收到数据，需要这个缓冲同步两个通道的数据长度。
    stereo_sync_buffer: Option<StereoSyncBuffer<f32>>,
}
#[godot_api]
impl ClapOutputAudioPortAccess {
    pub fn new(
        channel_accesses: Box<[Gd<ClapOutputAudioChannelAccess>]>,
        audio_stream_generator: Gd<AudioStreamGenerator>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            channel_accesses,
            stereo_sync_buffer: None,
            audio_stream_generator,
            audio_stream_generator_playback_weak_refs: Array::new(),
        })
    }

    fn audio_stream_generator_playbacks(&mut self) -> Box<[Gd<AudioStreamGeneratorPlayback>]> {
        let audio_stream_generator_playbacks = self
            .audio_stream_generator_playback_weak_refs
            .iter_shared()
            .filter_map(|weak_ref| assert_from_weak_ref(&weak_ref))
            .collect::<Box<[Gd<AudioStreamGeneratorPlayback>]>>();

        self.audio_stream_generator_playback_weak_refs.clear();
        self.audio_stream_generator_playback_weak_refs.extend(
            audio_stream_generator_playbacks
                .iter()
                .filter_map(assert_to_weak_ref),
        );

        audio_stream_generator_playbacks
    }

    pub fn process(&mut self) -> Box<[Box<[Box<[f32]>]>]> {
        let buffers = self
            .channel_accesses
            .iter_mut()
            .map(|channel_access| channel_access.bind_mut().process())
            .collect::<Box<_>>();

        let Some(left_channel_buffer) = buffers.get(0) else {
            return Box::new([]);
        };
        let right_channel_buffer = buffers.get(1).unwrap_or(left_channel_buffer);

        let mut stereo_sync_buffer = self.stereo_sync_buffer.take();
        let frames = OnceCell::<PackedVector2Array>::new();
        for mut audio_stream_generator_playback in self.audio_stream_generator_playbacks() {
            let frames_available = audio_stream_generator_playback.get_frames_available() as usize;
            let frames_would_push = frames
                .get_or_init(|| {
                    get_frames(
                        &mut stereo_sync_buffer,
                        left_channel_buffer,
                        right_channel_buffer,
                        &mut self.stereo_sync_buffer,
                        (self.audio_stream_generator.get_mix_rate()
                            * self.audio_stream_generator.get_buffer_length())
                            as usize,
                    )
                })
                .subarray(0, frames_available);
            audio_stream_generator_playback.push_buffer(&frames_would_push);
        }

        buffers
    }

    /// 获取本端口中的所有通道访问句柄。
    #[func]
    fn channel_accesses(&self) -> Array<Gd<ClapOutputAudioChannelAccess>> {
        self.channel_accesses.iter().cloned().collect()
    }
}
#[godot_api]
impl IAudioStream for ClapOutputAudioPortAccess {
    fn is_monophonic(&self) -> bool {
        self.channel_accesses.len() < 2
    }
    fn instantiate_playback(&self) -> Option<Gd<AudioStreamPlayback>> {
        let Some(audio_stream_generator_playback) =
            self.audio_stream_generator.clone().instantiate_playback()
        else {
            return None;
        };

        let Some(audio_stream_generator_playback_weak_ref) =
            assert_to_weak_ref(&audio_stream_generator_playback)
        else {
            return None;
        };

        self.audio_stream_generator_playback_weak_refs
            .clone()
            .push(&audio_stream_generator_playback_weak_ref);

        Some(audio_stream_generator_playback)
    }
}

enum StereoSyncBuffer<F> {
    Left(Vec<F>),
    Right(Vec<F>),
}
impl<F> StereoSyncBuffer<F> {
    fn into_stereo_buffers(self) -> (Box<[F]>, Box<[F]>) {
        match self {
            StereoSyncBuffer::Left(items) => (items.into_boxed_slice(), Box::new([])),
            StereoSyncBuffer::Right(items) => (Box::new([]), items.into_boxed_slice()),
        }
    }
}
impl<F> Deref for StereoSyncBuffer<F> {
    type Target = Vec<F>;
    fn deref(&self) -> &Self::Target {
        match self {
            StereoSyncBuffer::Left(items) => items,
            StereoSyncBuffer::Right(items) => items,
        }
    }
}

fn get_frames(
    stereo_sync_buffer: &mut Option<StereoSyncBuffer<f32>>,
    left_channel_buffer: &[Box<[f32]>],
    right_channel_buffer: &[Box<[f32]>],
    out_stereo_sync_buffer: &mut Option<StereoSyncBuffer<f32>>,
    out_stereo_sync_buffer_max_len: usize,
) -> PackedVector2Array {
    let mut frames = Vec::with_capacity(
        stereo_sync_buffer
            .as_ref()
            .map_or(0, |stereo_buffer| stereo_buffer.len())
            + left_channel_buffer.len().max(right_channel_buffer.len()),
    );

    let (left_sync_buffer, right_sync_buffer) = stereo_sync_buffer
        .take()
        .map(|stereo_buffer| stereo_buffer.into_stereo_buffers())
        .unwrap_or((Box::new([]), Box::new([])));
    let mut left_channel_buffer = left_sync_buffer
        .into_iter()
        .chain(left_channel_buffer.iter().flatten().cloned())
        .peekable();
    let mut right_channel_buffer = right_sync_buffer
        .into_iter()
        .chain(right_channel_buffer.iter().flatten().cloned())
        .peekable();

    *out_stereo_sync_buffer = loop {
        match (left_channel_buffer.peek(), right_channel_buffer.peek()) {
            (Some(left_channel_frame), Some(right_channel_frame)) => {
                frames.push(Vector2 {
                    x: *left_channel_frame,
                    y: *right_channel_frame,
                });
                left_channel_buffer.next();
                right_channel_buffer.next();
            }
            (Some(_), None) => {
                break Some(StereoSyncBuffer::Left(
                    left_channel_buffer
                        .take(out_stereo_sync_buffer_max_len)
                        .collect(),
                ));
            }
            (None, Some(_)) => {
                break Some(StereoSyncBuffer::Right(
                    right_channel_buffer
                        .take(out_stereo_sync_buffer_max_len)
                        .collect(),
                ));
            }
            (None, None) => break None,
        }
    };

    frames.into_iter().collect()
}
