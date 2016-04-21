
use nom::{IResult, ErrorKind};
use nom::Err::Position;
use nom::IResult::*;

use std::mem::transmute;

use super::Word;
use super::types::*;

named!(token<Word>, u16!(false));

pub fn opcode(input: &[u8], representation: Word) -> IResult<&[u8], &[u8]> {
    let bytes: [u8; 2] = unsafe { transmute(representation.to_le()) };
    tag!(input, bytes)
}

pub fn operand(input: &[u8]) -> IResult<&[u8], Operand> {
    match token(input) {
        Error(e)      => Error(e),
        Incomplete(i) => Incomplete(i),

        Done(rest, current) => {
            match current {
                0...32767 => {
                    Done(rest, Operand::Immediate(Immediate(current)))
                },
                32768...32775 => {
                    Done(rest, Operand::Register(Register(current - 32768)))
                },
                _ => Error(Position(ErrorKind::TagBits, input))
            }
        }
    }
}

macro_rules! instruction_0(
    ($i: expr, $opcode: expr, $instruction: ident, $semantics: ident) => {
        chain!($i, apply!(opcode, $opcode), ||
            Instruction::$instruction($semantics))
    }
);

macro_rules! instruction_1(
    ($i: expr, $opcode: expr, $instruction: ident, $semantics: ident) => {
        chain!($i, apply!(opcode, $opcode) ~ a: operand, ||
            Instruction::$instruction($semantics { a: a }))
    }
);

macro_rules! instruction_2(
    ($i: expr, $opcode: expr, $instruction: ident, $semantics: ident) => {
        chain!($i, apply!(opcode, $opcode) ~ a: operand ~ b: operand, ||
            Instruction::$instruction($semantics { a: a, b: b }))
    }
);

macro_rules! instruction_3(
    ($i: expr, $opcode: expr, $instruction: ident, $semantics: ident) => {
        chain!($i, apply!(opcode, $opcode) ~ a: operand ~ b: operand ~
            c: operand,
            || Instruction::$instruction($semantics { a: a, b: b, c: c }))
    }
);

named!(pub instruction<Instruction>, alt!(
    instruction_0!(0,  Halt, HaltSemantics) |
    instruction_2!(1,  Set, SetSemantics)   |
    instruction_1!(2,  Push, PushSemantics) |
    instruction_1!(3,  Pop, PopSemantics)   |
    instruction_3!(4,  Eq_, EqSemantics)    |
    instruction_3!(5,  Gt, GtSemantics)     |
    instruction_1!(6,  Jmp, JmpSemantics)   |
    instruction_2!(7,  Jt, JtSemantics)     |
    instruction_2!(8,  Jf, JfSemantics)     |
    instruction_3!(9,  Add, AddSemantics)   |
    instruction_3!(10, Mult, MultSemantics) |
    instruction_3!(11, Mod, ModSemantics)   |
    instruction_3!(12, And, AndSemantics)   |
    instruction_3!(13, Or, OrSemantics)     |
    instruction_2!(14, Not, NotSemantics)   |
    instruction_2!(15, Rmem, RmemSemantics) |
    instruction_2!(16, Wmem, WmemSemantics) |
    instruction_1!(17, Call, CallSemantics) |
    instruction_0!(18, Ret, RetSemantics)   |
    instruction_1!(19, Out, OutSemantics)   |
    instruction_1!(20, In, InSemantics)     |
    instruction_0!(21, Noop, NoopSemantics)
));

#[cfg(test)]
mod tests {
    use super::*;

    // Everything's awesome - except this.
    use super::super::types::{
        SetSemantics, Instruction, Operand, Register, Immediate
    };

    use nom::IResult::*;

    #[test]
    fn parse_immediate() {
        let bytes = [3u8, 0];
        let result = Operand::Immediate(Immediate(3));

        match operand(&bytes) {
            Done(tail, parsed) => {
                assert_eq!(parsed, result);
                assert!(tail.is_empty());
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_register() {
        let bytes = [2u8, 0x80];
        let result = Operand::Register(Register(2));

        match operand(&bytes) {
            Done(tail, parsed) => {
                assert_eq!(parsed, result);
                assert!(tail.is_empty());
            },
            _ => assert!(false)
        }
    }

    #[test]
    fn parse_set_instruction() {
        let bytes = [1u8, 0, 2, 0x80, 3, 0];
        let result = Instruction::Set(SetSemantics {
            a: Operand::Register(Register(2)),
            b: Operand::Immediate(Immediate(3))
        });

        match instruction(&bytes) {
            Done(tail, parsed) => {
                assert_eq!(parsed, result);
                assert!(tail.is_empty());
            },
            _ => panic!("Cannot parse.")
        }
    }
}
