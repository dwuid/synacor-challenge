
use std::io::Cursor;

use std;
use byteorder::{LittleEndian, ReadBytesExt};

use nom::IResult::*;

use super::Word;
use super::parser::instruction;
use super::types::{
    Operand, Instruction, State, Program, Semantics,
    REGISTER_COUNT, MEMORY_BITS
};

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

    let mut i = bytes.len();
    assert!(bytes.len() & 1 == 0, "Given byte array is of odd length.");

    while i > 0 {
        match reader.read_u16::<LittleEndian>() {
            Ok(word) => {
                result.push(word);
            },
            _ => panic!("Cannot convert byte-slice.")
        }

        i -= 2;
    }

    result
}

impl State {
    pub fn new(memory: &[u8]) -> Self {
        let words = to_u16(memory);

        let mut memory =  [0u16; (1 << MEMORY_BITS)];
        memory[..words.len()].clone_from_slice(&words);

        State {
            ip:     Some(0),
            memory: memory,
            .. Default::default()
        }
    }

    pub fn resolve_operand(&self, operand: &Operand) -> Word {
        match operand {
            &Operand::Register(ref index) => self.registers[index.0 as usize],
            &Operand::Immediate(ref immediate) => immediate.0,
        }
    }

    pub fn set_operand(&mut self, operand: &Operand, value: Word) {
        match operand {
            &Operand::Register(ref index) => {
                self.registers[index.0 as usize] = value;
            },
            _ => panic!("Cannot set non-register operand.")
        }
    }
}

impl Program {
    pub fn new(memory: &[u8]) -> Self {
        Program { state: State::new(memory) }
    }

    pub fn step(&mut self) -> bool {
        if self.state.ip.is_none() {
            return false;
        }

        let mut _opcodes = String::new();
        let instr: Instruction;
        let size: usize;

        {
            let ip = self.state.ip.unwrap() as usize;
            let words: &[u16]  = &self.state.memory[ip..];
            let current: &[u8] = unsafe { std::mem::transmute(words) };

            match instruction(&current) {
                Done(tail, parsed) => {
                    instr = parsed;
                    size = (current.len() - tail.len()) / 2;

                    for i in 0..size {
                        _opcodes.push_str(format!("{:04x} ",
                            words[i]).as_str());
                    }
                },

                _ => panic!("Cannot parse current instruction.")
            }
        }

        self.state.ip = self.state.ip.map(|ip| ip + size as u16);
        if self.state.ip.is_none() {
            panic!("Cannot compute on halted CPU.");
        }

        instr.apply(&mut self.state);
        true
    }
}
