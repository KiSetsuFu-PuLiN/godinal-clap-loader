use clack_host::{
    events::UnknownEvent,
    prelude::{InputEvents, OutputEvents},
};
use std::sync::mpsc::{Receiver, Sender};

pub struct EventBuffer<Handle> {
    event_buffer: clack_host::prelude::EventBuffer,
    handle: Handle,
}
impl<Handle> EventBuffer<Handle> {
    pub fn new(handle: Handle) -> Self {
        Self {
            event_buffer: clack_host::prelude::EventBuffer::new(),
            handle,
        }
    }
}

pub type InputHandle = Receiver<Box<[Box<UnknownEvent>]>>;
impl EventBuffer<InputHandle> {
    pub fn process(&mut self) {
        for event in self.handle.try_iter().flatten() {
            self.event_buffer.push(&event);
        }
    }
    pub fn pop_buffer(&self) -> InputEvents<'_> {
        self.event_buffer.as_input()
    }
}

pub type OutputHandle = Sender<Box<[Box<UnknownEvent>]>>;
impl EventBuffer<OutputHandle> {
    pub fn process(&mut self) {
        let events = self.event_buffer.iter().map(|event| {
            let event = event.as_bytes().to_vec().into_boxed_slice();
            let event = Box::into_raw(event);
            unsafe {
                let event = UnknownEvent::from_bytes_unchecked(&*event) as *const UnknownEvent
                    as *mut UnknownEvent;
                Box::from_raw(event as *mut UnknownEvent)
            }
        });
        self.handle.send(events.collect()).unwrap_or_else(|err|
            panic!(
                "音频事件缓冲输出通道寄了，ClapPluginInstance大概已经被销毁，本缓冲所在的线程应该也会很快销毁：{:?}", err) );
        self.event_buffer.clear();
    }
    pub fn pop_buffer(&mut self) -> OutputEvents<'_> {
        self.event_buffer.as_output()
    }
}
