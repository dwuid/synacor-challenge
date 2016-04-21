
use std;
use std::io::Read;
use std::num::Wrapping as W;

use super::Word;
use super::types::*;

fn normalize(value: Word) -> Word {
    value & ((1 << 15) - 1)
}

impl Semantics for NoopSemantics {
    fn apply(&self, _state: &mut State) {
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
        print!("{}", (state.resolve_operand(&self.a) as u8) as char);
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

        state.set_operand(&self.a, normalize((W(b) + W(c)).0))
    }
}

impl Semantics for MultSemantics {
    fn apply(&self, state: &mut State) {
        let b = state.resolve_operand(&self.b);
        let c = state.resolve_operand(&self.c);

        state.set_operand(&self.a, normalize((W(b) * W(c)).0))
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

impl Semantics for Instruction {
    fn apply(&self, mut state: &mut State) {
        use super::types::Instruction::*;

        match *self {
            Halt(ref semantics) => semantics.apply(state),
            Noop(ref semantics) => semantics.apply(state),
            Set(ref semantics)  => semantics.apply(state),
            Push(ref semantics) => semantics.apply(state),
            Pop(ref semantics)  => semantics.apply(state),
            Out(ref semantics)  => semantics.apply(state),
            In(ref semantics)   => semantics.apply(state),
            Eq_(ref semantics)  => semantics.apply(state),
            Gt(ref semantics)   => semantics.apply(state),
            Jmp(ref semantics)  => semantics.apply(state),
            Jt(ref semantics)   => semantics.apply(state),
            Jf(ref semantics)   => semantics.apply(state),
            Add(ref semantics)  => semantics.apply(state),
            Mult(ref semantics) => semantics.apply(state),
            Mod(ref semantics)  => semantics.apply(state),
            And(ref semantics)  => semantics.apply(state),
            Or(ref semantics)   => semantics.apply(state),
            Not(ref semantics)  => semantics.apply(state),
            Rmem(ref semantics) => semantics.apply(state),
            Wmem(ref semantics) => semantics.apply(state),
            Call(ref semantics) => semantics.apply(state),
            Ret(ref semantics)  => semantics.apply(state),
        }
    }
}
