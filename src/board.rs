use crate::device::{Attachment, Device};

struct GlobalState {
    seconds: u32,
    ticks: u32,
}

impl GlobalState {
    fn new() -> Self {
        GlobalState {
            seconds: 0,
            ticks: 0,
        }
    }
}

struct Board {
    state: GlobalState,
    devices: Vec<Box<dyn Device>>,
    wires: Vec<Attachment>,
}

impl Board {
    pub fn advance(&mut self) {}
}

