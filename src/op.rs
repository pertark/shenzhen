pub enum Register {}

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
}
