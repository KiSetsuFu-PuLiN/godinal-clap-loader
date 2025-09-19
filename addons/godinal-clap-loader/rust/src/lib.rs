mod clap_input_audio_channel_access;
mod clap_input_audio_port_access;
mod clap_output_audio_channel_access;
mod clap_output_audio_port_access;
mod clap_plugin_instance;
mod clap_transport_event_access;
mod host;
mod midi;
mod weak_ref;

use godot::init::{ExtensionLibrary, gdextension};

struct GodinalClapLoader;
#[gdextension]
unsafe impl ExtensionLibrary for GodinalClapLoader {}
