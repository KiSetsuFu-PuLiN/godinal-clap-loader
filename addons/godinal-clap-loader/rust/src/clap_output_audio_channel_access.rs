use crate::weak_ref::{assert_from_weak_ref, assert_to_weak_ref};
use godot::{
    classes::{
        AudioStreamGenerator, AudioStreamGeneratorPlayback, AudioStreamPlayback, IAudioStream,
        WeakRef,
    },
    prelude::*,
};
use std::{cell::OnceCell, sync::mpsc::Receiver};

/// 对Clap插件输出通道的访问句柄。
///
/// 可以被当作[`IAudioStream`]使用，通道输出的单声道音频会被复制并传入到两个输出声道中。
#[derive(GodotClass)]
#[class(no_init, base=AudioStream)]
pub struct ClapOutputAudioChannelAccess {
    channel_buffer_rx: Receiver<Box<[f32]>>,
    audio_stream_generator: Gd<AudioStreamGenerator>,
    audio_stream_generator_playback_weak_refs: Array<Gd<WeakRef>>,
}
#[godot_api]
impl ClapOutputAudioChannelAccess {
    pub fn new(
        channel_buffer_rx: Receiver<Box<[f32]>>,
        audio_stream_generator: Gd<AudioStreamGenerator>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            channel_buffer_rx,
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

    pub fn process(&mut self) -> Box<[Box<[f32]>]> {
        let buffer = self.channel_buffer_rx.try_iter().collect::<Box<_>>();

        let frames = OnceCell::<PackedVector2Array>::new();
        for mut audio_stream_generator_playback in self.audio_stream_generator_playbacks() {
            let frames_available = audio_stream_generator_playback.get_frames_available() as usize;
            let frames_would_push = frames
                .get_or_init(|| {
                    buffer
                        .iter()
                        .flatten()
                        .map(|value| Vector2::new(*value, *value))
                        .collect()
                })
                .subarray(0, frames_available);
            audio_stream_generator_playback.push_buffer(&frames_would_push);
        }

        buffer
    }
}
#[godot_api]
impl IAudioStream for ClapOutputAudioChannelAccess {
    fn is_monophonic(&self) -> bool {
        true
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
