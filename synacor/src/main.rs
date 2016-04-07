
#[macro_use]
extern crate nom;
extern crate byteorder;

use std::mem::transmute;
use std::io::{Read, Cursor};
use std::collections::HashMap;

use byteorder::{BigEndian, ReadBytesExt};

use nom::{IResult, ErrorKind};
use nom::Err::Position;
use nom::IResult::*;

const REGISTER_COUNT: usize = 8;
const MEMORY_BITS:    usize = 15;

pub type Word = u16;

#[derive(Debug, PartialEq)] pub struct Register(Word);
#[derive(Debug, PartialEq)] pub struct Immediate(Word);

#[derive(Debug, PartialEq)] pub enum Operand {
    Register(Register),
    Immediate(Immediate),
}

fn normalize(value: Word) -> Word {
    value & ((1 << 15) - 1)
}

struct State {
    ip:        Option<Word>,
    registers: [Word; REGISTER_COUNT],
    memory:    [Word; (1 << MEMORY_BITS)],
    stack:     Vec<Word>
}

impl Default for State {
    fn default() -> Self {
        State {
            ip:        None,
            registers: [0; REGISTER_COUNT],
            memory:    [0; (1 << MEMORY_BITS)],
            stack:     Vec::new()
        }
    }
}

fn to_u16(bytes: &[u8]) -> Vec<u16> {
    let mut reader = Cursor::new(bytes);
    let mut result = Vec::new();

    let mut i = 0;

    loop {
        match reader.read_u16::<BigEndian>() {
            Ok(word) => {
                println!("{} {}", i, word);
                result.push(word);
            },
            _ => panic!("Cannot convert byte-slice.")
        }

        i += 2;
    }

    result
}

impl State {
    fn new(memory: &[u8]) -> Self {
        let mut m =  [0u16; (1 << MEMORY_BITS)];

        let words = to_u16(memory);
        m.clone_from_slice(&words);

        State {
            ip:     Some(0),
            memory: m,
            .. Default::default()
        }
    }

    fn resolve_operand(&self, operand: &Operand) -> Word {
        match operand {
            &Operand::Register(ref index) => self.registers[index.0 as usize],
            &Operand::Immediate(ref immediate) => immediate.0,
        }
    }

    fn set_operand(&mut self, operand: &Operand, value: Word) {
        match operand {
            &Operand::Register(ref index) => {
                self.registers[index.0 as usize] = value;
            },
            _ => panic!("Cannot set non-register operand.")
        }
    }
}

struct Program {
    state: State
}

impl Program {
    fn new(memory: &[u8]) -> Self {
        Program { state: State::new(memory) }
    }
}

trait Semantics {
    fn apply(&self, state: &mut State);
}

#[derive(Debug, PartialEq)] pub struct HaltSemantics;
#[derive(Debug, PartialEq)] pub struct NoopSemantics;
#[derive(Debug, PartialEq)] pub struct SetSemantics { a: Operand, b: Operand }
#[derive(Debug, PartialEq)] pub struct PushSemantics { a: Operand }
#[derive(Debug, PartialEq)] pub struct PopSemantics { a: Operand }
#[derive(Debug, PartialEq)] pub struct OutSemantics { a: Operand }
#[derive(Debug, PartialEq)] pub struct InSemantics { a: Operand }

#[derive(Debug, PartialEq)] pub struct EqSemantics {
    a: Operand, b: Operand, c: Operand
}

#[derive(Debug, PartialEq)] pub struct GtSemantics {
    a: Operand, b: Operand, c: Operand
}

#[derive(Debug, PartialEq)] pub struct JmpSemantics { a: Operand }
#[derive(Debug, PartialEq)] pub struct JtSemantics { a: Operand, b: Operand }
#[derive(Debug, PartialEq)] pub struct JfSemantics { a: Operand, b: Operand }

#[derive(Debug, PartialEq)] pub struct AddSemantics {
    a: Operand, b: Operand, c: Operand
}

#[derive(Debug, PartialEq)] pub struct MultSemantics {
    a: Operand, b: Operand, c: Operand
}

#[derive(Debug, PartialEq)] pub struct ModSemantics {
    a: Operand, b: Operand, c: Operand
}

#[derive(Debug, PartialEq)] pub struct AndSemantics {
    a: Operand, b: Operand, c: Operand
}

#[derive(Debug, PartialEq)] pub struct OrSemantics {
    a: Operand, b: Operand, c: Operand
}

#[derive(Debug, PartialEq)] pub struct NotSemantics { a: Operand, b: Operand }
#[derive(Debug, PartialEq)] pub struct RmemSemantics { a: Operand, b: Operand }
#[derive(Debug, PartialEq)] pub struct WmemSemantics { a: Operand, b: Operand }
#[derive(Debug, PartialEq)] pub struct CallSemantics { a: Operand }
#[derive(Debug, PartialEq)] pub struct RetSemantics;

// On Semantics::apply, state.ip already points to the following instruction.

impl Semantics for NoopSemantics {
    fn apply(&self, state: &mut State) {
    }
}

impl Semantics for HaltSemantics {
    fn apply(&self, state: &mut State) {
        state.ip = None;
    }
}

impl Semantics for SetSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        state.set_operand(&self.a, b);
    }
}

impl Semantics for PushSemantics {
    fn apply(&self, state: &mut State) {
        let a = state.resolve_operand(&self.a);
        state.stack.push(a);
    }
}

impl Semantics for PopSemantics {
    fn apply(&self, state: &mut State) {
        let tos = state.stack.pop().expect("Cannot pop from empty stack.");
        state.set_operand(&self.a, tos);
    }
}

impl Semantics for OutSemantics {
    fn apply(&self, state: &mut State) {
        print!("{}", state.resolve_operand(&self.a));
    }
}

impl Semantics for InSemantics {
    fn apply(&self, state: &mut State) {
        let input: Option<u16> = std::io::stdin().bytes().next()
            .and_then(|r| r.ok()).map(|b| b as u16);

        if let Some(input) = input {
            state.set_operand(&self.a, input);
            return;
        }

        panic!("Invalid input.");
    }
}

impl Semantics for EqSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let c = state.resolve_operand(&self.c);

        state.set_operand(&self.a, if b == c {
            1
        } else {
            0
        });
    }
}


impl Semantics for GtSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let c = state.resolve_operand(&self.c);

        state.set_operand(&self.a, if b > c {
            1
        } else {
            0
        });
    }
}

impl Semantics for JmpSemantics {
    fn apply(&self, state: &mut State) {
        let a = state.resolve_operand(&self.a);
        state.ip = Some(a);
    }
}

impl Semantics for JtSemantics {
    fn apply(&self, state: &mut State) {
        let a = state.resolve_operand(&self.a);
        let b = state.resolve_operand(&self.b);
        if a != 0 {
            state.ip = Some(b);
        }
    }
}

impl Semantics for JfSemantics {
    fn apply(&self, state: &mut State) {
        let a = state.resolve_operand(&self.a);
        let b = state.resolve_operand(&self.b);
        if a == 0 {
            state.ip = Some(b);
        }
    }
}

impl Semantics for AddSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let c = state.resolve_operand(&self.c);

        state.set_operand(&self.a, normalize(b + c))
    }
}

impl Semantics for MultSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let c = state.resolve_operand(&self.c);

        state.set_operand(&self.a, normalize(b * c))
    }
}

impl Semantics for ModSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let c = state.resolve_operand(&self.c);

        state.set_operand(&self.a, normalize(b % c))
    }
}

impl Semantics for AndSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let c = state.resolve_operand(&self.c);

        state.set_operand(&self.a, normalize(b & c))
    }
}

impl Semantics for OrSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let c = state.resolve_operand(&self.c);

        state.set_operand(&self.a, normalize(b | c))
    }
}

impl Semantics for NotSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        state.set_operand(&self.a, normalize(!b))
    }
}

impl Semantics for RmemSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let v = state.memory[b as usize];
        state.set_operand(&self.a, v);
    }
}

impl Semantics for WmemSemantics {
    fn apply(&self, state: &mut State) {
        let a = state.resolve_operand(&self.a);
        let b = state.resolve_operand(&self.b);
        state.memory[a as usize] = b;
    }
}

impl Semantics for CallSemantics {
    fn apply(&self, state: &mut State) {
        if let Some(ip) = state.ip {
            state.stack.push(ip);
            state.ip = Some(state.resolve_operand(&self.a));
            return;
        }

        panic!("Current ip is unknown.");
    }
}

impl Semantics for RetSemantics {
    fn apply(&self, state: &mut State) {
        let tos = state.stack.pop();
        state.ip = tos;
    }
}

#[derive(Debug, PartialEq)]
pub enum Instruction {
    Halt(HaltSemantics),
    Noop(NoopSemantics),
    Set(SetSemantics),
    Push(PushSemantics),
    Pop(PopSemantics),
    Out(OutSemantics),
    In(InSemantics),
    Eq_(EqSemantics),
    Gt(GtSemantics),
    Jmp(JmpSemantics),
    Jt(JtSemantics),
    Jf(JfSemantics),
    Add(AddSemantics),
    Mult(MultSemantics),
    Mod(ModSemantics),
    And(AndSemantics),
    Or(OrSemantics),
    Not(NotSemantics),
    Rmem(RmemSemantics),
    Wmem(WmemSemantics),
    Call(CallSemantics),
    Ret(RetSemantics),
}

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

named!(pub program_parser<Vec<Instruction> >, many0!(instruction));

fn parser(input: &[u8]) -> Option<HashMap<usize, Instruction> > {
    let mut result = HashMap::new();
    let mut cursor = input;
    let mut offset = 0;

    while cursor.len() > 0 {
        match instruction(cursor) {
            Done(tail, current) => {
                offset = input.len() - cursor.len();
                println!("{} {} {:?}", offset, cursor.len(), current);
                cursor = tail;

                result.insert(offset, current);
            },

            _ => return None
        }
    }

    Some(result)
}

fn parse_program(input: &[u8]) -> Option<Vec<Instruction>> {
    match program_parser(input) {
        Done(tail, result) => {
            println!("{:x}", input.len() - tail.len());
            for i in 0..100 {
                print!("{:x} ", tail[i]);
            }
            Some(result)
        },
        _ => None
    }
}

fn main() {
    let input = include_bytes!("../challenge.bin");
    let program = Program::new(input);
    /*if let Some(instructions) = parse_program(input) {
        /*for (i, instruction) in instructions.iter().enumerate() {
            println!("{} {:?}", i, instruction);
        }*/
    }

    if let Some(instructions) = parser(input) {
        for (offset, instruction) in &instructions {
            println!("{} {:?}", offset, instructions);
        }
    } else {
        println!("Cannot parse.");
    }*/
}

#[cfg(test)]
mod tests {
    use super::*;
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

