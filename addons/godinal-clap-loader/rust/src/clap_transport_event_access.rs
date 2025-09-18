use clack_host::{
    events::{
        EventFlags, EventHeader,
        event_types::{TransportEvent, TransportFlags},
    },
    utils::{BeatTime, SecondsTime},
};
use godot::prelude::*;

/// 用于和插件同步宿主状态的类，当其被赋给`ClapPluginInstance`的`clap_transport_event_access`属性后，就可以用于操作同步该Clap插件实例的宿主信息。
///
/// todo: 还有几个bool的Flag没有暴露控制。
///
/// todo: 成员都还没有写注释。
#[derive(GodotClass)]
#[class(no_init)]
pub struct ClapTransportEventAccess {
    base: Base<RefCounted>,
    transport_event: TransportEvent,
}
#[godot_api]
impl ClapTransportEventAccess {
    #[func]
    fn new(
        header_time: u32,
        song_pos_beats: i64,
        song_pos_seconds: i64,
        tempo: f64,
        tempo_inc: f64,
        loop_start_beats: i64,
        loop_end_beats: i64,
        loop_start_seconds: i64,
        loop_end_seconds: i64,
        bar_start: i64,
        bar_number: i32,
        time_signature_numerator: i16,
        time_signature_denominator: i16,
    ) -> Gd<Self> {
        Gd::from_init_fn(move |base| Self {
            base,
            transport_event: TransportEvent {
                // 本来这两个Flag也想注册到参数的，但是参数太长了gdext不给干。
                header: EventHeader::new_core(header_time, EventFlags::empty()),
                flags: TransportFlags::empty(),
                song_pos_beats: BeatTime::from_int(song_pos_beats),
                song_pos_seconds: SecondsTime::from_int(song_pos_seconds),
                tempo,
                tempo_inc,
                loop_start_beats: BeatTime::from_int(loop_start_beats),
                loop_end_beats: BeatTime::from_int(loop_end_beats),
                loop_start_seconds: SecondsTime::from_int(loop_start_seconds),
                loop_end_seconds: SecondsTime::from_int(loop_end_seconds),
                bar_start: BeatTime::from_int(bar_start),
                bar_number,
                time_signature_numerator,
                time_signature_denominator,
            },
        })
    }

    #[signal]
    pub fn value_changed();

    pub fn transport_event(&self) -> &TransportEvent {
        &self.transport_event
    }

    #[func]
    fn get_header_time(&self) -> u32 {
        self.transport_event.header.time()
    }
    #[func]
    fn set_header_time(&mut self, header_time: u32) {
        self.transport_event.header.set_time(header_time);
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_song_pos_beats(&self) -> i64 {
        self.transport_event.song_pos_beats.to_int()
    }
    #[func]
    fn set_song_pos_beats(&mut self, song_pos_beats: i64) {
        self.transport_event.song_pos_beats = BeatTime::from_int(song_pos_beats);
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_song_pos_seconds(&self) -> i64 {
        self.transport_event.song_pos_seconds.to_int()
    }
    #[func]
    fn set_song_pos_seconds(&mut self, song_pos_seconds: i64) {
        self.transport_event.song_pos_seconds = SecondsTime::from_int(song_pos_seconds);
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_tempo(&self) -> f64 {
        self.transport_event.tempo
    }
    #[func]
    fn set_tempo(&mut self, tempo: f64) {
        self.transport_event.tempo = tempo;
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_tempo_inc(&self) -> f64 {
        self.transport_event.tempo_inc
    }
    #[func]
    fn set_tempo_inc(&mut self, tempo_inc: f64) {
        self.transport_event.tempo_inc = tempo_inc;
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_loop_start_beats(&self) -> i64 {
        self.transport_event.loop_start_beats.to_int()
    }
    #[func]
    fn set_loop_start_beats(&mut self, loop_start_beats: i64) {
        self.transport_event.loop_start_beats = BeatTime::from_int(loop_start_beats);
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_loop_end_beats(&self) -> i64 {
        self.transport_event.loop_end_beats.to_int()
    }
    #[func]
    fn set_loop_end_beats(&mut self, loop_end_beats: i64) {
        self.transport_event.loop_end_beats = BeatTime::from_int(loop_end_beats);
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_loop_start_seconds(&self) -> i64 {
        self.transport_event.loop_start_seconds.to_int()
    }
    #[func]
    fn set_loop_start_seconds(&mut self, loop_start_seconds: i64) {
        self.transport_event.loop_start_seconds = SecondsTime::from_int(loop_start_seconds);
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_loop_end_seconds(&self) -> i64 {
        self.transport_event.loop_end_seconds.to_int()
    }
    #[func]
    fn set_loop_end_seconds(&mut self, loop_end_seconds: i64) {
        self.transport_event.loop_end_seconds = SecondsTime::from_int(loop_end_seconds);
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_bar_start(&self) -> i64 {
        self.transport_event.bar_start.to_int()
    }
    #[func]
    fn set_bar_start(&mut self, bar_start: i64) {
        self.transport_event.bar_start = BeatTime::from_int(bar_start);
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_bar_number(&self) -> i32 {
        self.transport_event.bar_number
    }
    #[func]
    fn set_bar_number(&mut self, bar_number: i32) {
        self.transport_event.bar_number = bar_number;
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_time_signature_numerator(&self) -> i16 {
        self.transport_event.time_signature_numerator
    }
    #[func]
    fn set_time_signature_numerator(&mut self, time_signature_numerator: i16) {
        self.transport_event.time_signature_numerator = time_signature_numerator;
        self.signals().value_changed().emit();
    }

    #[func]
    fn get_time_signature_denominator(&self) -> i16 {
        self.transport_event.time_signature_denominator
    }
    #[func]
    fn set_time_signature_denominator(&mut self, time_signature_denominator: i16) {
        self.transport_event.time_signature_denominator = time_signature_denominator;
        self.signals().value_changed().emit();
    }
}
