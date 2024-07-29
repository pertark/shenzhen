use crate::{
    code::{Loc, RegImm, Register},
    devices::mc::execute_loc,
};

pub trait Device {
    fn step(&mut self);
}

pub trait McDevice {
    fn read_reg(&self, reg: Register) -> i16;
    fn write_reg(&mut self, reg: Register, val: i16);
    fn set_condition(&mut self, state: CondState);
    fn get_state(&self) -> DeviceState;
    fn sleep(&mut self, duration: u32);
    fn get_loc(&mut self) -> Option<Loc>;
    fn update_attachments(&mut self);
    fn read_reg_or_imm(&mut self, reg_imm: RegImm) -> i16 {
        match reg_imm {
            RegImm::Imm(imm) => imm,
            RegImm::Reg(reg) => self.read_reg(reg),
        }
    }

    fn read_two_regimm(&mut self, r1: RegImm, r2: RegImm) -> (i16, i16) {
        (self.read_reg_or_imm(r1), self.read_reg_or_imm(r2))
    }
}

// we need some form of updating attached devices
// since the devices are mmaped to registers [reg] -> attachment
// after a cycle of updates, we should "apply" those updates by taking mapped registers and writing
// to their corresponding attachments

impl<T: McDevice> Device for T {
    fn step(&mut self) {
        match self.get_state() {
            DeviceState::Exec => {
                if let Some(code) = self.get_loc() {
                    execute_loc(code, self);
                }
            }
            DeviceState::Sleep(_) => {
                todo!() // sleeping for n time units
            }
            DeviceState::Write => {
                todo!() // blocking, waiting for write
            }
            DeviceState::Read => {
                todo!() // blocking, waiting for read
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum DeviceState {
    Sleep(u32),
    Exec,
    Write,
    Read,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CondState {
    None,
    Plus,
    Minus,
}

pub struct Code<const N: usize> {
    code: [Loc; N],
    pc: usize,
    pub state: CondState,
}

impl<const N: usize> Code<N> {
    pub fn get_next_line(&mut self) -> Option<Loc> {
        for _ in 0..N {
            let line = self.code[self.pc].clone();

            match line.op.clone() {
                None => self.pc += 1,
                Some(_) => match line.cond {
                    None => {
                        return Some(line);
                    }
                    Some(cond) => {
                        todo!()
                    }
                },
            }
        }

        None
    }
}

#[derive(Copy, Clone)]
pub struct SimpleIOWire {
    value: i16,
}

impl SimpleIOWire {
    pub fn push_update_with(&mut self, data: i16) {
        todo!();
    }
}

#[derive(Copy, Clone)]
pub struct XBusWire {
    value: Option<i16>,
}

pub enum Attachment {
    SimpleIO(SimpleIOWire),
    XBus(XBusWire),
}

impl Attachment {
    fn read_value_from_attachment(&self) -> Option<i16> {
        match self {
            Attachment::SimpleIO(x) => Some(x.value),
            Attachment::XBus(x) => x.value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    //     #[test]
    //     fn test_regimm_to_imm() {
    //         let mut device = MC4000 {
    //             acc: 0,
    //             state: DeviceState::Exec,
    //             pins: [None, None, None, None],
    //             code: Code {
    //                 code: [(); 9].map(|_| Loc {
    //                     op: None,
    //                     lab: None,
    //                     cond: None,
    //                 }),
    //                 pc: 0,
    //                 state: CondState::None,
    //             },
    //         };
    //         device.set_acc(5);
    //         assert_eq!(regimm_to_imm(&mut device, RegImm::Imm(3)), 3);
    //         assert_eq!(regimm_to_imm(&mut device, RegImm::Reg(Register::Acc)), 5);
    //     }
    //
    //     #[test]
    //     fn test_read_reg() {
    //         let siowire = Rc::new(RefCell::new(SimpleIOWire { value: 4 }));
    //         let xbwire = Rc::new(RefCell::new(XBusWire {
    //             value: Some(7),
    //             timeout: 0,
    //         }));
    //         let mut device = MC4000 {
    //             acc: 0,
    //             state: DeviceState::Exec,
    //             pins: [Some(siowire), None, Some(xbwire), None],
    //             code: Code {
    //                 code: [(); 9].map(|_| Loc {
    //                     op: None,
    //                     lab: None,
    //                     cond: None,
    //                 }),
    //                 pc: 0,
    //                 state: CondState::None,
    //             },
    //         };
    //         device.set_acc(5);
    //         assert_eq!(read_reg(&mut device, Register::Acc), 5);
    //         assert_eq!(read_reg(&mut device, Register::Pin(Pin::P0)), 4);
    //         assert_eq!(read_reg(&mut device, Register::Pin(Pin::X0)), 7);
    //     }
    //
    //     #[test]
    //     fn test_write_reg() {
    //         let siowire = Rc::new(RefCell::new(SimpleIOWire { value: 4 }));
    //         let xbwire = Rc::new(RefCell::new(XBusWire {
    //             value: Some(7),
    //             timeout: 0,
    //         }));
    //         let mut device = MC4000 {
    //             acc: 0,
    //             state: DeviceState::Exec,
    //             pins: [Some(siowire.clone()), None, Some(xbwire.clone()), None],
    //             code: Code {
    //                 code: [(); 9].map(|_| Loc {
    //                     op: None,
    //                     lab: None,
    //                     cond: None,
    //                 }),
    //                 pc: 0,
    //                 state: CondState::None,
    //             },
    //         };
    //         write_reg(&mut device, Register::Acc, 5);
    //         write_reg(&mut device, Register::Pin(Pin::P0), 5);
    //         write_reg(&mut device, Register::Pin(Pin::X0), 5);
    //         assert_eq!(device.get_acc(), Some(5));
    //         assert_eq!(siowire.borrow_mut().get(), 5);
    //         assert_eq!(xbwire.borrow_mut().get(), 5);
    //     }
    //
    //     // opcode tests
    //     #[test]
    //     fn test_nop() {
    //         let mut device = MC4000 {
    //             acc: 0,
    //             state: DeviceState::Exec,
    //             pins: [None, None, None, None],
    //             code: Code {
    //                 code: [(); 9].map(|_| Loc {
    //                     op: Some(Opcode::Nop),
    //                     lab: None,
    //                     cond: None,
    //                 }),
    //                 pc: 0,
    //                 state: CondState::None,
    //             },
    //         };
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 1);
    //     }
    //
    //     #[test]
    //     fn test_movreg() {
    //         let siowire = Rc::new(RefCell::new(SimpleIOWire { value: 4 }));
    //         let mut device = MC4000 {
    //             acc: 0,
    //             state: DeviceState::Exec,
    //             pins: [Some(siowire.clone()), None, None, None],
    //             code: Code {
    //                 code: [0, 1, 2, 3, 4, 5, 6, 7, 8].map(|i| Loc {
    //                     op: Some(Opcode::MovReg(RegImm::Imm(i), Register::Acc)),
    //                     lab: None,
    //                     cond: None,
    //                 }),
    //                 pc: 0,
    //                 state: CondState::None,
    //             },
    //         };
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 1);
    //         assert_eq!(device.get_acc(), Some(0));
    //
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 2);
    //         assert_eq!(device.get_acc(), Some(1));
    //
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 3);
    //         assert_eq!(device.get_acc(), Some(2));
    //
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 4);
    //         assert_eq!(device.get_acc(), Some(3));
    //
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 5);
    //         assert_eq!(device.get_acc(), Some(4));
    //
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 6);
    //         assert_eq!(device.get_acc(), Some(5));
    //
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 7);
    //         assert_eq!(device.get_acc(), Some(6));
    //
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 8);
    //         assert_eq!(device.get_acc(), Some(7));
    //
    //         exec(device.code.step().unwrap(), &mut device);
    //         assert_eq!(device.code.pc, 0);
    //         assert_eq!(device.get_acc(), Some(8));
    //     }
}
