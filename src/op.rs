pub enum Pin {
    P0,
    P1,
    X0,
    X1,
    X2,
    X3,
}

pub enum Register {
    Acc,
    Dat,
    Pin(Pin),
}

pub struct Label(String);

pub struct Condition(bool);

pub enum RegImm {
    Reg(Register),
    Imm(u16),
}

pub enum Opcode {
    Nop,
    MovReg(RegImm, Register),
    Jmp(u8),
    Slp(RegImm),
    Add(RegImm),
    Sub(RegImm),
    Mul(RegImm),
    Not,
    Dgt(RegImm),
    Dst(RegImm, RegImm),
    Teq(RegImm, RegImm),
    Tgt(RegImm, RegImm),
    Tlt(RegImm, RegImm),
    Tcp(RegImm, RegImm),
}

pub struct LOC {
    cond: Option<Condition>,
    lab: Option<Label>,
    op: Opcode,
}
