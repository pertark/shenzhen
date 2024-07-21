use crate::op::{Pin, LOC};

pub trait Device {
    fn step(&self);
    fn get_pin(&self, pin: Pin) -> Option<Box<dyn Device>>;
}

#[derive(Clone, Copy)]
pub enum DeviceState {
    SLEEP,
    EXEC,
    WRITE,
    READ,
}

struct MC4000 {
    acc: u16,
    state: DeviceState,
    pins: [Option<Box<dyn Device>>; 4], 
    pc: u8,
    code: [LOC; 9],
}

impl Device for MC4000 {
    fn step(&self) {
        
    } 
    fn get_pin(&self, pin: Pin) -> Option<Box<dyn Device>> {
        match pin {
            Pin::P0 => self.pins[0],
            Pin::P1 => self.pins[1],
            Pin::X0 => self.pins[2],
            Pin::X1 => self.pins[3],
            _ => None,
        }.clone()
    }
}