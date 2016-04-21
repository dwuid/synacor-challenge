
use super::Word;

pub const REGISTER_COUNT: usize = 8;
pub const MEMORY_BITS:    usize = 15;

#[derive(Debug, PartialEq)] pub struct Register(pub Word);
#[derive(Debug, PartialEq)] pub struct Immediate(pub Word);

#[derive(Debug, PartialEq)] pub enum Operand {
    Register(Register),
    Immediate(Immediate),
}

pub struct State {
    pub ip:        Option<Word>,
    pub registers: [Word; REGISTER_COUNT],
    pub memory:    [Word; (1 << MEMORY_BITS)],
    pub stack:     Vec<Word>
}

pub struct Program {
    pub state: State
}

pub trait Semantics {
    fn apply(&self, state: &mut State);
}

#[derive(Debug, PartialEq)] pub struct HaltSemantics;
#[derive(Debug, PartialEq)] pub struct NoopSemantics;
#[derive(Debug, PartialEq)] pub struct SetSemantics {
    pub a: Operand, pub b: Operand
}

#[derive(Debug, PartialEq)] pub struct PushSemantics { pub a: Operand }
#[derive(Debug, PartialEq)] pub struct PopSemantics { pub a: Operand }
#[derive(Debug, PartialEq)] pub struct OutSemantics { pub a: Operand }
#[derive(Debug, PartialEq)] pub struct InSemantics { pub a: Operand }

#[derive(Debug, PartialEq)] pub struct EqSemantics {
    pub a: Operand, pub b: Operand, pub c: Operand
}

#[derive(Debug, PartialEq)] pub struct GtSemantics {
    pub a: Operand, pub b: Operand, pub c: Operand
}

#[derive(Debug, PartialEq)] pub struct JmpSemantics { pub a: Operand }
#[derive(Debug, PartialEq)] pub struct JtSemantics {
    pub a: Operand, pub b: Operand
}

#[derive(Debug, PartialEq)] pub struct JfSemantics {
    pub a: Operand, pub b: Operand
}

#[derive(Debug, PartialEq)] pub struct AddSemantics {
    pub a: Operand, pub b: Operand, pub c: Operand
}

#[derive(Debug, PartialEq)] pub struct MultSemantics {
    pub a: Operand, pub b: Operand, pub c: Operand
}

#[derive(Debug, PartialEq)] pub struct ModSemantics {
    pub a: Operand, pub b: Operand, pub c: Operand
}

#[derive(Debug, PartialEq)] pub struct AndSemantics {
    pub a: Operand, pub b: Operand, pub c: Operand
}

#[derive(Debug, PartialEq)] pub struct OrSemantics {
    pub a: Operand, pub b: Operand, pub c: Operand
}

#[derive(Debug, PartialEq)] pub struct NotSemantics {
    pub a: Operand, pub b: Operand
}

#[derive(Debug, PartialEq)] pub struct RmemSemantics {
    pub a: Operand, pub b: Operand
}

#[derive(Debug, PartialEq)] pub struct WmemSemantics {
    pub a: Operand, pub b: Operand
}

#[derive(Debug, PartialEq)] pub struct CallSemantics { pub a: Operand }
#[derive(Debug, PartialEq)] pub struct RetSemantics;

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
