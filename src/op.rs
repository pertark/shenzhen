use std::str::FromStr;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_till},
    character::complete::{self, alphanumeric1},
    combinator::{map, value},
    multi::many_m_n,
    sequence::{terminated, tuple},
    Finish, IResult,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Register {
    Acc,
    Dat,
    Pin(Pin),
}

impl Register {
    pub fn lex_from_str(input: &str) -> IResult<&str, Self> {
        alt((
            value(Register::Acc, tag("acc")),
            value(Register::Dat, tag("dat")),
            |inp| {
                let (r, x) = Pin::lex_from_str(inp)?;
                Ok((r, Register::Pin(x)))
            },
        ))(input)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Label(String);

impl Label {
    pub fn lex_from_str(input: &str) -> IResult<&str, Self> {
        let (remaining, string): (&str, &str) =
            terminated(take_till(|c| c == ':'), tag(":"))(input)?;
        Ok((remaining, Label(String::from(string))))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Condition(bool);

impl Condition {
    pub fn lex_from_str(input: &str) -> IResult<&str, Self> {
        alt((
            value(Condition(true), tag("+")),
            value(Condition(false), tag("-")),
        ))(input)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum RegImm {
    Reg(Register),
    Imm(u16),
}

impl RegImm {
    pub fn lex_from_str(input: &str) -> IResult<&str, Self> {
        alt((
            map(Register::lex_from_str, RegImm::Reg),
            map(complete::u16, RegImm::Imm),
        ))(input)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
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

impl Opcode {
    pub fn lex_from_str(input: &str) -> IResult<&str, Self> {
        alt((
            value(Opcode::Nop, tag("nop")),
            map(
                tuple((
                    tag("mov"),
                    tag(" "),
                    RegImm::lex_from_str,
                    tag(" "),
                    Register::lex_from_str,
                )),
                |(_, _, i, _, o)| Opcode::MovReg(i, o),
            ),
            map(
                tuple((
                    tag("jmp"),
                    tag(" "),
                    map(alphanumeric1, |x| Label(String::from(x))),
                )),
                |(_, _, l)| Opcode::Jmp(l),
            ),
            map(
                tuple((tag("slp"), tag(" "), RegImm::lex_from_str)),
                |(_, _, r)| Opcode::Slp(r),
            ),
            map(
                tuple((tag("add"), tag(" "), RegImm::lex_from_str)),
                |(_, _, r)| Opcode::Add(r),
            ),
            map(
                tuple((tag("sub"), tag(" "), RegImm::lex_from_str)),
                |(_, _, r)| Opcode::Sub(r),
            ),
            map(
                tuple((tag("mul"), tag(" "), RegImm::lex_from_str)),
                |(_, _, r)| Opcode::Mul(r),
            ),
            value(Opcode::Not, tag("not")),
            map(
                tuple((tag("dgt"), tag(" "), RegImm::lex_from_str)),
                |(_, _, r)| Opcode::Dgt(r),
            ),
            map(
                tuple((
                    tag("dst"),
                    tag(" "),
                    RegImm::lex_from_str,
                    tag(" "),
                    RegImm::lex_from_str,
                )),
                |(_, _, i, _, o)| Opcode::Dst(i, o),
            ),
            map(
                tuple((
                    tag("teq"),
                    tag(" "),
                    RegImm::lex_from_str,
                    tag(" "),
                    RegImm::lex_from_str,
                )),
                |(_, _, i, _, o)| Opcode::Teq(i, o),
            ),
            map(
                tuple((
                    tag("tgt"),
                    tag(" "),
                    RegImm::lex_from_str,
                    tag(" "),
                    RegImm::lex_from_str,
                )),
                |(_, _, i, _, o)| Opcode::Tgt(i, o),
            ),
            map(
                tuple((
                    tag("tlt"),
                    tag(" "),
                    RegImm::lex_from_str,
                    tag(" "),
                    RegImm::lex_from_str,
                )),
                |(_, _, i, _, o)| Opcode::Tlt(i, o),
            ),
            map(
                tuple((
                    tag("tcp"),
                    tag(" "),
                    RegImm::lex_from_str,
                    tag(" "),
                    RegImm::lex_from_str,
                )),
                |(_, _, i, _, o)| Opcode::Tcp(i, o),
            ),
        ))(input)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LOC {
    cond: Option<Condition>,
    lab: Option<Label>,
    op: Option<Opcode>,
}

impl LOC {
    pub fn lex_from_str(input: &str) -> IResult<&str, Self> {
        map(
            tuple((
                map(
                    many_m_n(
                        0,
                        1,
                        map(tuple((Label::lex_from_str, tag(" "))), |(a, _)| a),
                    ),
                    |x| x.first().cloned(),
                ),
                map(
                    many_m_n(
                        0,
                        1,
                        map(tuple((Condition::lex_from_str, tag(" "))), |(a, _)| a),
                    ),
                    |x| x.first().cloned(),
                ),
                map(many_m_n(0, 1, Opcode::lex_from_str), |x| x.first().cloned()),
            )),
            |(l, c, o)| Self {
                cond: c,
                lab: l,
                op: o,
            },
        )(input)
    }
}

impl FromStr for LOC {
    type Err = nom::error::Error<String>;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match LOC::lex_from_str(s).finish() {
            Ok((_remaining, loc)) => Ok(loc),
            Err(nom::error::Error { input, code }) => Err(nom::error::Error {
                input: input.to_string(),
                code,
            }),
        }
    }
}
