use std::{cell::RefCell, rc::Rc};

use crate::op::{Condition, Pin, LOC};


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

#[derive(PartialEq)]
pub enum CondState {
    None,
    Plus,
    Minus,
    All
}

struct Code<const N: usize> {
    code: [LOC; N],
    pc: u8,
    pub state: CondState
}

impl<const N: usize> Code<N> {
    fn step(&mut self) -> Option<LOC> {
        let mut i = 0;
        loop {
            let line = self.code[self.pc as usize].clone();
            if line.op == None {
                self.pc += 1;
            } else if let Some(cond) = line.cond {
                match self.state {
                    CondState::All => break,
                    CondState::Plus => {
                        if cond == Condition(true) {
                            break;
                        } else {
                            self.pc += 1;
                        }
                    },
                    CondState::Minus => {
                        if cond == Condition(false) {
                            break;
                        } else {
                            self.pc += 1;
                        }
                    }
                    CondState::None => self.pc += 1,
                }
            }
            self.pc = self.pc % N as u8;
            i += 1;
            if i >= N {
                return None;
            }
        }

        Some(self.code[self.pc as usize].clone())
    }
}

#[derive(PartialEq)]
enum Protocol {
    XBus,
    SimpleIO
}

struct Wire {
    value: u16,
    protocol: Protocol,
    pins: [Option<Rc<RefCell<dyn Device>>>; 16],
}

impl Device for Wire {
    fn step(&mut self) {
        if self.protocol == Protocol::XBus {
            panic!("blocked on read");
        }
    }
    fn get_pin(&self, pin: Pin) -> Option<Rc<RefCell<dyn Device>>> {
        None 
    }
}

struct MC4000 {
    acc: u16,
    state: DeviceState,
    pins: [Option<Rc<RefCell<dyn Device>>>; 4],
    code: Code<9>,
}

impl Device for MC4000 {
    fn step(&mut self) {
        if let Some(code) = self.code.step() {
            // there are intermediates... 
        }
    }
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

