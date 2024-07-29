use std::{cell::RefCell, rc::Rc};

use crate::op::{Condition, Opcode, Pin, RegImm, Register, LOC};

// maybe should split MC devices into another trait
pub trait Device {
    fn step(&mut self);
    fn get_pin(&self, pin: Pin) -> Option<Rc<RefCell<dyn Wire>>>;
    fn set_pin(&self, pin: Pin, val: i16);
    fn get_acc(&self) -> Option<i16>;
    fn set_acc(&mut self, val: i16);
    fn get_dat(&self) -> Option<i16>;
    fn set_dat(&mut self, val: i16);
    fn set_cnd(&mut self, cnd: CondState);
    fn sleep(&mut self, duration: i16);
}

#[derive(Clone, Copy)]
pub enum DeviceState {
    Sleep(u32),
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

pub trait Wire {
    fn get(&mut self) -> i16; // needs to be mut for XBus
    fn push(&mut self, value: i16);
    fn step(&mut self);
}
struct SimpleIOWire {
    value: i16,
}

impl Wire for SimpleIOWire {
    fn get(&mut self) -> i16 { self.value }
    fn push(&mut self, value: i16) {
        self.value = value; 
    }
    fn step(&mut self) { }
}

struct XBusWire {
    value: Option<i16>,
    // src: Option<Rc<RefCell<dyn Device>>> ???
    timeout: u8,
}

impl Wire for XBusWire {
    fn get(&mut self) -> i16 {
        if let Some(v) = self.value {
            self.value = None;
            return v;
        } else {
            panic!("part blocked on read");
        }
    }
    fn push(&mut self, value: i16) {
        self.value = Some(value);
        self.timeout = 0;
    }
    fn step(&mut self) {
        if self.value != None {
            // idk. maybe you get a couple timesteps of leeway before you get part blocked on write
            // TODO: check this timeout value
            if self.timeout > 10 {
                panic!("Part blocked on write")
            } else { self.timeout += 1 }
        }
    }
}

struct MC4000 {
    acc: i16,
    state: DeviceState,
    pins: [Option<Rc<RefCell<dyn Wire>>>; 4],
    pin_vals: [Option<i16>; 4],
    code: Code<9>,
}

impl Device for MC4000 {
    fn step(&mut self) {
        match self.state {
            DeviceState::Exec => {
                if let Some(code) = self.code.step() {
                    exec(code, self);
                }
            },
            // Read and Write are XBus operations.
            _ => {}
        }
    }
    fn get_pin(&self, pin: Pin) -> Option<Rc<RefCell<dyn Wire>>> {
        match pin {
            Pin::P0 => self.pins[0].clone(),
            Pin::P1 => self.pins[1].clone(),
            Pin::X0 => self.pins[2].clone(),
            Pin::X1 => self.pins[3].clone(),
            _ => None,
        }
    }
    fn set_pin(&self, pin: Pin, val: i16) {
        let i = match pin {
            Pin::P0 => 0,
            Pin::P1 => 1,
            Pin::X0 => 2,
            Pin::X1 => 3,
            _ => panic!("bad pin"),
        };
    }
    fn get_acc(&self) -> Option<i16> {
        Some(self.acc)
    }
    fn set_acc(&mut self, val: i16) {
        self.acc = val;
    }
    fn get_dat(&self) -> Option<i16> {
        None
    }
    fn set_dat(&mut self, _val: i16) {
        panic!("bad")
    }
    fn set_cnd(&mut self, cnd: CondState) {
        self.code.state = cnd;
    }
    fn sleep(&mut self, duration: i16) {
        let t = if duration < 0 { 0 } else { duration as u32 };
        self.state = DeviceState::Sleep(t);
    }
}

fn exec(loc: LOC, device: &mut dyn Device) {
    let op = loc.op.unwrap();
    match op {
        Opcode::Nop => {},
        Opcode::MovReg(reg_imm, reg) => { 
            let src = match reg_imm {
                RegImm::Reg(reg) => read_reg(device, reg),
                RegImm::Imm(imm) => imm,
            };
            write_reg(device, reg, src);
        },
        Opcode::Jmp(label) => {
            // TODO: jump to label
        },
        Opcode::Slp(reg_imm) => {
            // TODO: wait til start of next second. or nth second
            // there are actually an infinite number of timesteps between
            // this second and the next second, but for practical reasons,
            // we can put a hard cap and throw a part not sleeping error.
            let val = regimm_to_imm(device, reg_imm);
            device.sleep(val)
        },
        Opcode::Add(reg_imm) => {
            let acc = device.get_acc().expect("bad");
            let next = acc + regimm_to_imm(device, reg_imm);
            device.set_acc(next)
        },
        Opcode::Sub(reg_imm) => {
            let acc = device.get_acc().expect("bad");
            let next = acc - regimm_to_imm(device, reg_imm);
            device.set_acc(next)
        },
        Opcode::Mul(reg_imm) => {
            let acc = device.get_acc().expect("bad");
            let next = acc * regimm_to_imm(device, reg_imm);
            device.set_acc(next)
        },
        Opcode::Not => {
            let acc = device.get_acc().expect("bad");
            let next = if acc == 0 { 100 } else { 0 };
            device.set_acc(next)
        },
        Opcode::Dgt(reg_imm) => {
            let acc = device.get_acc().expect("bad");
            let pos = regimm_to_imm(device, reg_imm) + 1;
            // need some value check
            let sign = acc.signum();
            let next = sign * (acc.abs() % 10i16.pow(pos as u32)) / 10i16.pow(pos as u32);
            device.set_acc(next)
        },
        Opcode::Dst(reg_imm, reg_imm2) => {
            let acc = device.get_acc().expect("bad");
            let pos = regimm_to_imm(device, reg_imm) + 1;
            let dgt = regimm_to_imm(device, reg_imm2); // TODO: check that it's between 0-9.
            let sign = acc.signum();
            let next = acc - sign * (acc.abs() % 10i16.pow(pos as u32)) 
                                + sign * (acc.abs() % 10i16.pow(pos as u32 - 1)) 
                                + sign * dgt * 10i16.pow(pos as u32);
            device.set_acc(next)
        },
        Opcode::Teq(reg_imm, reg_imm2) => {
            let lhs = regimm_to_imm(device, reg_imm);
            let rhs = regimm_to_imm(device, reg_imm2);
            if lhs == rhs {
                device.set_cnd(CondState::Plus) 
            } else {
                device.set_cnd(CondState::Minus)
            }
        },
        Opcode::Tgt(reg_imm, reg_imm2) => {
            let lhs = regimm_to_imm(device, reg_imm);
            let rhs = regimm_to_imm(device, reg_imm2);
            if lhs > rhs {
                device.set_cnd(CondState::Plus) 
            } else {
                device.set_cnd(CondState::Minus)
            }
        },
        Opcode::Tlt(reg_imm, reg_imm2) => {
            let lhs = regimm_to_imm(device, reg_imm);
            let rhs = regimm_to_imm(device, reg_imm2);
            if lhs < rhs {
                device.set_cnd(CondState::Plus) 
            } else {
                device.set_cnd(CondState::Minus)
            }
        },
        Opcode::Tcp(reg_imm, reg_imm2) => {
            let lhs = regimm_to_imm(device, reg_imm);
            let rhs = regimm_to_imm(device, reg_imm2);
            if lhs == rhs {
                device.set_cnd(CondState::None) 
            } else if lhs > rhs {
                device.set_cnd(CondState::Plus)
            } else {
                device.set_cnd(CondState::Minus)
            }
        },
    }
}

fn regimm_to_imm(device: &mut dyn Device, regimm: RegImm) -> i16 {
    match regimm {
        RegImm::Imm(imm) => imm,
        RegImm::Reg(reg) => read_reg(device, reg)
    }
}

fn read_reg(device: &mut dyn Device, reg: Register) -> i16 {
    match reg {
        Register::Acc => device.get_acc().expect("invalid register"),
        Register::Dat => device.get_dat().expect("invalid register"),
        Register::Pin(pin) => device.get_pin(pin).expect("not connected").borrow_mut().get(),
    }
}

fn write_reg(device: &mut dyn Device, reg: Register, val: i16) {
    match reg {
        Register::Acc => device.set_acc(val),
        Register::Dat => device.set_dat(val),
        Register::Pin(pin) => device.get_pin(pin).expect("not connected").borrow_mut().push(val),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regimm_to_imm() {
        let mut device = MC4000 {
            acc: 0,
            state: DeviceState::Exec,
            pins: [None, None, None, None],
            code: Code {
                code: [(); 9].map(|_| LOC { op: None, lab: None, cond: None }),
                pc: 0,
                state: CondState::None,
            },
        };
        device.set_acc(5);
        assert_eq!(regimm_to_imm(&mut device, RegImm::Imm(3)), 3);
        assert_eq!(regimm_to_imm(&mut device, RegImm::Reg(Register::Acc)), 5);
    }

    #[test]
    fn test_read_reg() {
        let siowire = Rc::new(RefCell::new(SimpleIOWire { value: 4 }));
        let xbwire = Rc::new(RefCell::new(XBusWire { value: Some(7), timeout: 0 }));
        let mut device = MC4000 {
            acc: 0,
            state: DeviceState::Exec,
            pins: [Some(siowire), None, Some(xbwire), None],
            code: Code {
                code: [(); 9].map(|_| LOC { op: None, lab: None, cond: None }),
                pc: 0,
                state: CondState::None,
            },
        };
        device.set_acc(5);
        assert_eq!(read_reg(&mut device, Register::Acc), 5);
        assert_eq!(read_reg(&mut device, Register::Pin(Pin::P0)), 4);
        assert_eq!(read_reg(&mut device, Register::Pin(Pin::X0)), 7);
    }

    #[test]
    fn test_write_reg() {
        let siowire = Rc::new(RefCell::new(SimpleIOWire { value: 4 }));
        let xbwire = Rc::new(RefCell::new(XBusWire { value: Some(7), timeout: 0 }));
        let mut device = MC4000 {
            acc: 0,
            state: DeviceState::Exec,
            pins: [Some(siowire.clone()), None, Some(xbwire.clone()), None],
            code: Code {
                code: [(); 9].map(|_| LOC { op: None, lab: None, cond: None }),
                pc: 0,
                state: CondState::None,
            },
        };
        write_reg(&mut device, Register::Acc, 5);
        write_reg(&mut device, Register::Pin(Pin::P0), 5);
        write_reg(&mut device, Register::Pin(Pin::X0), 5);
        assert_eq!(device.get_acc(), Some(5));
        assert_eq!(siowire.borrow_mut().get(), 5);
        assert_eq!(xbwire.borrow_mut().get(), 5);
    }

    // opcode tests
    #[test]
    fn test_nop() {
        let mut device = MC4000 {
            acc: 0,
            state: DeviceState::Exec,
            pins: [None, None, None, None],
            code: Code {
                code: [(); 9].map(|_| LOC { op: Some(Opcode::Nop), lab: None, cond: None }),
                pc: 0,
                state: CondState::None,
            },
        };
        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 1);
    }

    #[test]
    fn test_movreg() {
        let siowire = Rc::new(RefCell::new(SimpleIOWire { value: 4 }));
        let mut device = MC4000 {
            acc: 0,
            state: DeviceState::Exec,
            pins: [Some(siowire.clone()), None, None, None],
            code: Code {
                code: [0,1,2,3,4,5,6,7,8].map(|i| LOC { op: Some(Opcode::MovReg(RegImm::Imm(i), Register::Acc)), lab: None, cond: None }),
                pc: 0,
                state: CondState::None,
            },
        };
        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 1);
        assert_eq!(device.get_acc(), Some(0));

        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 2);
        assert_eq!(device.get_acc(), Some(1));

        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 3);
        assert_eq!(device.get_acc(), Some(2));

        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 4);
        assert_eq!(device.get_acc(), Some(3));

        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 5);
        assert_eq!(device.get_acc(), Some(4));

        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 6);
        assert_eq!(device.get_acc(), Some(5));

        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 7);
        assert_eq!(device.get_acc(), Some(6));

        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 8);
        assert_eq!(device.get_acc(), Some(7));

        exec(device.code.step().unwrap(), &mut device);
        assert_eq!(device.code.pc, 0);
        assert_eq!(device.get_acc(), Some(8));
    }
}
