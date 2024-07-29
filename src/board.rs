use crate::device::{Device, Wire};

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
    wires: Vec<Box<dyn Wire>>,
}

impl Board {
    pub fn advance(&mut self) {
        while (self.devices.iter().all(|d| d))
        for device in self.devices.iter_mut() {
            device.step();
        }
    }
}