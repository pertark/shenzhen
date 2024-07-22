use std::{cell::RefCell, rc::Rc};

use crate::op::{Pin, LOC};

pub trait Device {
    fn step(&mut self);
    fn get_pin(&self, pin: Pin) -> Option<Rc<RefCell<dyn Device>>>;
}

#[derive(Clone, Copy)]
pub enum DeviceState {
    Sleep,
    Exec,
    Write,
    Read,
}

struct MC4000 {
    acc: u16,
    state: DeviceState,
    pins: [Option<Rc<RefCell<dyn Device>>>; 4],
    pc: u8,
    code: [LOC; 9],
}

impl Device for MC4000 {
    fn step(&mut self) {}
    fn get_pin(&self, pin: Pin) -> Option<Rc<RefCell<dyn Device>>> {
        match pin {
            Pin::P0 => self.pins[0].clone(),
            Pin::P1 => self.pins[1].clone(),
            Pin::X0 => self.pins[2].clone(),
            Pin::X1 => self.pins[3].clone(),
            _ => None,
        }
    }
}

