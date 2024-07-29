use crate::{
    code::{Loc, Opcode, Pin::*, Register},
    device::{Code, CondState, DeviceState, McDevice, SimpleIOWire, XBusWire},
};

pub fn execute_loc(loc: Loc, device: &mut dyn McDevice) {
    let op = loc.op.unwrap();
    match op {
        Opcode::Nop => {}
        Opcode::MovReg(r1, r2) => {
            let out = device.read_reg_or_imm(r1);
            device.write_reg(r2, out);
        }
        Opcode::Jmp(label) => {
            // TODO: jump to label
        }
        Opcode::Slp(r) => {
            // TODO: wait til start of next second. or nth second
            // there are actually an infinite number of timesteps between
            // this second and the next second, but for practical reasons,
            // we can put a hard cap and throw a part not sleeping error.
            let out = device.read_reg_or_imm(r).try_into().unwrap();
            device.sleep(out);
        }
        Opcode::Add(r) => {
            let out = device.read_reg(Register::Acc) + device.read_reg_or_imm(r);
            device.write_reg(Register::Acc, out)
        }
        Opcode::Sub(r) => {
            let out = device.read_reg(Register::Acc) - device.read_reg_or_imm(r);
            device.write_reg(Register::Acc, out)
        }
        Opcode::Mul(r) => {
            let out = device.read_reg(Register::Acc) * device.read_reg_or_imm(r);
            device.write_reg(Register::Acc, out)
        }
        Opcode::Not => {
            let out = if device.read_reg(Register::Acc) == 0 {
                100
            } else {
                0
            };
            device.write_reg(Register::Acc, out)
        }
        Opcode::Dgt(r) => {
            let pos = device.read_reg_or_imm(r) + 1;
            let acc = device.read_reg(Register::Acc);
            // need some value check
            let sign = acc.signum();
            let next = sign * (acc.abs() % 10i16.pow(pos as u32)) / 10i16.pow(pos as u32);
            device.write_reg(Register::Acc, next)
        }
        Opcode::Dst(r1, r2) => {
            let acc = device.read_reg(Register::Acc);
            let pos = device.read_reg_or_imm(r1) + 1;
            let dgt = device.read_reg_or_imm(r2); // TODO: check that it's between 0-9.
            let sign = acc.signum();
            let next = acc - sign * (acc.abs() % 10i16.pow(pos as u32))
                + sign * (acc.abs() % 10i16.pow(pos as u32 - 1))
                + sign * dgt * 10i16.pow(pos as u32);
            device.write_reg(Register::Acc, next)
        }
        Opcode::Teq(r1, r2) => match device.read_two_regimm(r1, r2) {
            (x, y) if x == y => device.set_condition(CondState::Plus),
            (_, _) => device.set_condition(CondState::Minus),
        },
        Opcode::Tgt(r1, r2) => match device.read_two_regimm(r1, r2) {
            (x, y) if x > y => device.set_condition(CondState::Plus),
            (_, _) => device.set_condition(CondState::Minus),
        },

        Opcode::Tlt(r1, r2) => match device.read_two_regimm(r1, r2) {
            (x, y) if x < y => device.set_condition(CondState::Plus),
            (_, _) => device.set_condition(CondState::Minus),
        },
        Opcode::Tcp(r1, r2) => match device.read_two_regimm(r1, r2) {
            (x, y) if x == y => device.set_condition(CondState::None),
            (x, y) if x > y => device.set_condition(CondState::Plus),
            (_, _) => device.set_condition(CondState::Minus),
        },
    }
}

pub struct MC4000 {
    regs: [i16; 5],
    state: DeviceState,
    code: Code<9>,
    attachments: (
        Option<SimpleIOWire>,
        Option<SimpleIOWire>,
        Option<XBusWire>,
        Option<XBusWire>,
    ),
}

impl McDevice for MC4000 {
    fn read_reg(&self, reg: Register) -> i16 {
        match reg {
            Register::Acc => self.regs[0],
            Register::Dat => panic!("dat does not exist"),
            Register::Pin(p) => match p {
                P0 => self.regs[1],
                P1 => self.regs[2],
                X0 => self.regs[3],
                X1 => self.regs[4],
                X2 => panic!("x2 does not exist"),
                X3 => panic!("x3 does not exist"),
            },
        }
    }

    fn set_condition(&mut self, state: crate::device::CondState) {
        self.code.state = state;
    }

    fn sleep(&mut self, duration: u32) {
        self.state = DeviceState::Sleep(duration);
    }

    fn write_reg(&mut self, reg: Register, val: i16) {
        match reg {
            Register::Acc => self.regs[0] = val,
            Register::Dat => panic!("dat does not exist"),
            Register::Pin(p) => match p {
                P0 => self.regs[1] = val,
                P1 => self.regs[2] = val,
                X0 => self.regs[3] = val,
                X1 => self.regs[4] = val,
                X2 => panic!("x2 does not exist"),
                X3 => panic!("x3 does not exist"),
            },
        };
    }

    fn get_state(&self) -> DeviceState {
        self.state
    }

    fn get_loc(&mut self) -> Option<Loc> {
        self.code.get_next_line()
    }

    fn update_attachments(&mut self) {
        if let Some(mut x) = self.attachments.0 {
            x.push_update_with(self.regs[1]);
        }
        if let Some(mut x) = self.attachments.1 {
            x.push_update_with(self.regs[2]);
        }
        if let Some(mut _x) = self.attachments.2 {
            todo!()
        }
        if let Some(mut _x) = self.attachments.3 {
            todo!()
        }
    }
}

pub struct MC4000X {
    regs: [i16; 5],
    state: DeviceState,
    code: Code<9>,
    attachments: (
        Option<XBusWire>,
        Option<XBusWire>,
        Option<XBusWire>,
        Option<XBusWire>,
    ),
}

impl McDevice for MC4000X {
    fn read_reg(&self, reg: Register) -> i16 {
        match reg {
            Register::Acc => self.regs[0],
            Register::Dat => panic!("dat does not exist"),
            Register::Pin(p) => match p {
                P0 => panic!("p0 does not exist"),
                P1 => panic!("p1 does not exist"),
                X0 => self.regs[1],
                X1 => self.regs[2],
                X2 => self.regs[3],
                X3 => self.regs[4],
            },
        }
    }

    fn set_condition(&mut self, state: crate::device::CondState) {
        self.code.state = state;
    }

    fn sleep(&mut self, duration: u32) {
        self.state = DeviceState::Sleep(duration);
    }

    fn write_reg(&mut self, reg: Register, val: i16) {
        match reg {
            Register::Acc => self.regs[0] = val,
            Register::Dat => panic!("dat does not exist"),
            Register::Pin(p) => match p {
                P0 => panic!("p0 does not exist"),
                P1 => panic!("p1 does not exist"),
                X0 => self.regs[1] = val,
                X1 => self.regs[2] = val,
                X2 => self.regs[3] = val,
                X3 => self.regs[4] = val,
            },
        };
    }

    fn get_state(&self) -> DeviceState {
        self.state
    }

    fn get_loc(&mut self) -> Option<Loc> {
        self.code.get_next_line()
    }

    fn update_attachments(&mut self) {
        todo!()
    }
}

pub struct MC6000 {
    regs: [i16; 8],
    state: DeviceState,
    code: Code<15>, // who knows tbh
    attachments: ([Option<SimpleIOWire>; 2], [Option<XBusWire>; 4]),
}

impl McDevice for MC6000 {
    fn read_reg(&self, reg: Register) -> i16 {
        match reg {
            Register::Acc => self.regs[0],
            Register::Dat => self.regs[1],
            Register::Pin(p) => match p {
                P0 => self.regs[2],
                P1 => self.regs[3],
                X0 => self.regs[4],
                X1 => self.regs[5],
                X2 => self.regs[6],
                X3 => self.regs[7],
            },
        }
    }

    fn set_condition(&mut self, state: crate::device::CondState) {
        self.code.state = state;
    }

    fn sleep(&mut self, duration: u32) {
        self.state = DeviceState::Sleep(duration);
    }

    fn write_reg(&mut self, reg: Register, val: i16) {
        match reg {
            Register::Acc => self.regs[0] = val,
            Register::Dat => self.regs[1] = val,
            Register::Pin(p) => match p {
                P0 => self.regs[2] = val,
                P1 => self.regs[3] = val,
                X0 => self.regs[4] = val,
                X1 => self.regs[5] = val,
                X2 => self.regs[6] = val,
                X3 => self.regs[7] = val,
            },
        }
    }

    fn get_state(&self) -> DeviceState {
        self.state
    }

    fn get_loc(&mut self) -> Option<Loc> {
        self.code.get_next_line()
    }

    fn update_attachments(&mut self) {
        todo!()
    }
}
