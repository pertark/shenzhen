use std::str::FromStr;

use nom::{branch::alt, bytes::complete::tag, combinator::value, IResult};

#[derive(Copy, Clone)]
pub enum Pin {
    P0,
    P1,
    X0,
    X1,
    X2,
    X3,
}

impl Pin {
    pub fn lex_from_str(input: &str) -> IResult<&str, Self> {
        alt((
            value(Pin::P0, tag("p0")),
            value(Pin::P1, tag("p1")),
            value(Pin::X0, tag("x0")),
            value(Pin::X1, tag("x1")),
            value(Pin::X2, tag("x2")),
            value(Pin::X3, tag("x3")),
        ))(input)
    }
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
    Jmp(Label),
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
