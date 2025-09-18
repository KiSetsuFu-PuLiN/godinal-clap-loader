use clack_host::{
    events::{
        Event, Pckn, UnknownEvent,
        event_types::{MidiSysExEvent, NoteOffEvent, NoteOnEvent, ParamValueEvent},
        spaces::CoreEventSpace,
    },
    utils::{ClapId, Cookie},
};
use godot::{
    classes::InputEventMidi,
    global::{MidiMessage, godot_warn},
    obj::{Gd, NewGd},
};

fn to_unknown<E: Event>(event: E) -> Box<UnknownEvent> {
    let event = Box::new(event);
    let event_prt = event.as_unknown() as *const UnknownEvent as *mut UnknownEvent;
    let _ = Box::into_raw(event);
    unsafe { Box::from_raw(event_prt) }
}

fn assert_default_midi(event: Box<UnknownEvent>) -> Gd<InputEventMidi> {
    godot_warn!("尚未实现转换 Clap Event: {:?}", event);
    InputEventMidi::new_gd()
}

// todo: 完善下述映射。

pub fn midi_to_event(midi: Gd<InputEventMidi>) -> Box<UnknownEvent> {
    let time = 0;
    let velocity = midi.get_velocity() as f64 / 127.0;
    let param_id = ClapId::from_raw(midi.get_controller_number() as u32);
    let port_index = midi.get_device() as u16;
    let pckn = Pckn::new(
        port_index,
        midi.get_channel() as u16,
        midi.get_pitch() as u16,
        midi.instance_id().to_i64() as u32,
    );
    let value = midi.get_controller_value() as f64 / 127.0;
    let cookie = Cookie::empty();

    match midi.get_message() {
        MidiMessage::NONE => todo!(),
        MidiMessage::NOTE_OFF => to_unknown(NoteOffEvent::new(time, pckn, velocity)),
        MidiMessage::NOTE_ON => to_unknown(NoteOnEvent::new(time, pckn, velocity)),
        MidiMessage::AFTERTOUCH => todo!(),
        MidiMessage::CONTROL_CHANGE => to_unknown(ParamValueEvent::new(
            time,
            param_id.unwrap(),
            pckn,
            value,
            cookie,
        )),
        MidiMessage::PROGRAM_CHANGE => todo!(),
        MidiMessage::CHANNEL_PRESSURE => todo!(),
        MidiMessage::PITCH_BEND => todo!(),
        MidiMessage::SYSTEM_EXCLUSIVE => to_unknown(MidiSysExEvent::new(time, port_index, &[])),
        MidiMessage::QUARTER_FRAME => todo!(),
        MidiMessage::SONG_POSITION_POINTER => todo!(),
        MidiMessage::SONG_SELECT => todo!(),
        MidiMessage::TUNE_REQUEST => todo!(),
        MidiMessage::TIMING_CLOCK => todo!(),
        MidiMessage::START => todo!(),
        MidiMessage::CONTINUE => todo!(),
        MidiMessage::STOP => todo!(),
        MidiMessage::ACTIVE_SENSING => todo!(),
        MidiMessage::SYSTEM_RESET => todo!(),
        _ => unreachable!("未处理的Midi事件：{:?}", midi.get_message()),
    }
}

pub fn event_to_midi(event: Box<UnknownEvent>) -> Gd<InputEventMidi> {
    let Some(core_event) = event.as_core_event() else {
        return assert_default_midi(event);
    };

    let mut midi = InputEventMidi::new_gd();
    match core_event {
        CoreEventSpace::NoteOn(note_on_event) => {
            midi.set_velocity((note_on_event.velocity() * 127.0) as i32);
            midi.set_device(note_on_event.port_index().to_raw() as i32);
            midi.set_channel(note_on_event.channel().to_raw() as i32);
            midi.set_pitch(note_on_event.key().to_raw() as i32);
            midi
        }
        CoreEventSpace::NoteOff(note_off_event) => {
            midi.set_velocity((note_off_event.velocity() * 127.0) as i32);
            midi.set_device(note_off_event.port_index().to_raw() as i32);
            midi.set_channel(note_off_event.channel().to_raw() as i32);
            midi.set_pitch(note_off_event.key().to_raw() as i32);
            midi
        }
        CoreEventSpace::NoteChoke(note_choke_event) => assert_default_midi(event),
        CoreEventSpace::NoteEnd(note_end_event) => assert_default_midi(event),
        CoreEventSpace::NoteExpression(note_expression_event) => assert_default_midi(event),
        CoreEventSpace::ParamValue(param_value_event) => {
            midi.set_controller_number(
                param_value_event
                    .param_id()
                    .map_or(0, |clap_id| clap_id.get() as i32),
            );
            midi.set_device(param_value_event.port_index().to_raw() as i32);
            midi.set_channel(param_value_event.channel().to_raw() as i32);
            midi.set_pitch(param_value_event.key().to_raw() as i32);
            midi.set_controller_value((param_value_event.value() * 127.0) as i32);
            midi
        }
        CoreEventSpace::ParamMod(param_mod_event) => assert_default_midi(event),
        CoreEventSpace::ParamGestureBegin(param_gesture_begin_event) => assert_default_midi(event),
        CoreEventSpace::ParamGestureEnd(param_gesture_end_event) => assert_default_midi(event),
        CoreEventSpace::Transport(transport_event) => assert_default_midi(event),
        CoreEventSpace::Midi(midi_event) => assert_default_midi(event),
        CoreEventSpace::Midi2(midi2_event) => assert_default_midi(event),
        CoreEventSpace::MidiSysEx(midi_sys_ex_event) => {
            midi.set_device(midi_sys_ex_event.port_index() as i32);
            midi
        }
    }
}
