use crate::{
    clap_input_audio_port_access::ClapInputAudioPortAccess,
    clap_output_audio_port_access::ClapOutputAudioPortAccess,
    host::audio_processor::audio_buffer::channel::{InputChannel, OutputChannel},
};
use clack_host::prelude::AudioPortBufferType;
use godot::{classes::AudioStreamGenerator, prelude::*};
use std::{array::IntoIter, iter::repeat_n};

pub struct Port<Channel> {
    channels: Box<[Channel]>,
}

impl<F> Port<InputChannel<F>> {
    pub fn process(&mut self) {
        for channel_buffer in &mut self.channels {
            channel_buffer.process();
        }
    }
}
impl Port<InputChannel<f32>> {
    pub fn new(
        channels_count: usize,
        frames_count: usize,
        sample_rate: f64,
    ) -> (Self, Gd<ClapInputAudioPortAccess>) {
        let (channels, clap_input_audio_channel_accesses): (Vec<_>, Vec<_>) =
            repeat_n((), channels_count)
                .map(|()| InputChannel::<f32>::new(frames_count))
                .unzip();
        (
            Self {
                channels: channels.into(),
            },
            ClapInputAudioPortAccess::new(sample_rate, clap_input_audio_channel_accesses.into()),
        )
    }

    pub fn pop_buffer(
        &mut self,
        frames_count: usize,
    ) -> clack_host::prelude::AudioPortBuffer<
        impl IntoIterator<Item = clack_host::prelude::InputChannel<'_, f32>>,
        IntoIter<clack_host::prelude::InputChannel<'_, f64>, 0>,
    > {
        clack_host::prelude::AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_input_only(self.channels.iter_mut().map(
                move |audio_channel_buffer| audio_channel_buffer.pop_buffer(frames_count, 0.0),
            )),
        }
    }
}
#[deprecated(note = "在以Cardinal.clap进行开发测试时，发现f64将导致段错误。故不要使用这个实现。")]
#[allow(unused)]
impl Port<InputChannel<f64>> {
    pub fn pop_buffer(
        &mut self,
        frames_count: usize,
    ) -> clack_host::prelude::AudioPortBuffer<
        IntoIter<clack_host::prelude::InputChannel<'_, f32>, 0>,
        impl IntoIterator<Item = clack_host::prelude::InputChannel<'_, f64>>,
    > {
        clack_host::prelude::AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f64_input_only(self.channels.iter_mut().map(
                move |audio_channel_buffer| audio_channel_buffer.pop_buffer(frames_count, 0.0),
            )),
        }
    }
}

impl<F> Port<OutputChannel<F>> {
    pub fn process(&mut self) {
        for channel_buffer in &mut self.channels {
            channel_buffer.process();
        }
    }
}
impl Port<OutputChannel<f32>> {
    pub fn new(
        channels_count: usize,
        audio_stream_generator: Gd<AudioStreamGenerator>,
    ) -> (Self, Gd<ClapOutputAudioPortAccess>) {
        let (channels, clap_output_audio_channel_access): (Vec<_>, Vec<_>) =
            repeat_n((), channels_count)
                .map(|()| OutputChannel::<f32>::new(audio_stream_generator.clone()))
                .unzip();
        (
            Self {
                channels: channels.into(),
            },
            ClapOutputAudioPortAccess::new(
                clap_output_audio_channel_access.into(),
                audio_stream_generator,
            ),
        )
    }

    pub fn pop_buffer(
        &mut self,
        frames_count: usize,
    ) -> clack_host::prelude::AudioPortBuffer<
        impl IntoIterator<Item = &mut [f32]>,
        IntoIter<&'_ mut [f64], 0>,
    > {
        clack_host::prelude::AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f32_output_only(
                self.channels
                    .iter_mut()
                    .map(move |channel_buffer| channel_buffer.pop_buffer(frames_count, 0.0)),
            ),
        }
    }
}
#[deprecated(note = "在以Cardinal.clap进行开发测试时，发现f64将导致段错误。故不要使用这个实现。")]
#[allow(unused)]
impl Port<OutputChannel<f64>> {
    pub fn pop_buffer(
        &mut self,
        frames_count: usize,
    ) -> clack_host::prelude::AudioPortBuffer<
        IntoIter<&'_ mut [f32], 0>,
        impl IntoIterator<Item = &mut [f64]>,
    > {
        clack_host::prelude::AudioPortBuffer {
            latency: 0,
            channels: AudioPortBufferType::f64_output_only(
                self.channels
                    .iter_mut()
                    .map(move |channel_buffer| channel_buffer.pop_buffer(frames_count, 0.0)),
            ),
        }
    }
}
