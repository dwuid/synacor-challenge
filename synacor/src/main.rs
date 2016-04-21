
#[macro_use]
extern crate nom;
extern crate byteorder;

mod interpreter;
use interpreter::Program;

fn main() {
    let input = include_bytes!("../../challenge/challenge.bin");
    let mut program = Program::new(input);

    while program.step() {
    }
}
