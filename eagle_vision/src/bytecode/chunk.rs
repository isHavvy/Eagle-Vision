use bevy::prelude::*;

use crate::bytecode::{Value, opcode::Opcode};

/// A chunk of opcodes and associated constants.
/// 
/// Limitation: There is a hard limit of 256 constants per chunk.
#[derive(Debug, Clone, Default, Component)]
pub struct Chunk {
    // Invariant: code.len() == node.len()

    /// Opcode and their operands
    code: Vec<u8>,

    /// Associated source expression
    source: Vec<Entity>,

    /// Constant data used by this chunk
    consts: Vec<Value>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            source: vec![],

            consts: vec![]
        }
    }

    pub fn push_opcode(&mut self, op: Opcode, source: Entity) {
        self.push_code(op as u8, source);
    }

    pub fn push_code(&mut self, code: u8, source: Entity) {
        self.code.push(code);
        self.source.push(source);
    }

    #[inline]
    pub fn index_code(&self, ix: usize) -> u8 {
        self.code[ix]
    }

    #[inline]
    pub fn index_consts(&mut self, ix: usize) -> Value {
        self.consts[ix].clone()
    }

    pub fn push_const(&mut self, value: Value) -> Result<u8, TooManyConstantsError> {
        self.consts.push(value);
        (self.consts.len() - 1).try_into().map_err(|_| TooManyConstantsError)
    }

    // pub fn disassemble(&self, name: Cow<'_, str>) -> Result<String, std::fmt::Error> {
    //     use std::fmt::Write;

    //     let mut output = String::new();
    //     writeln!(&mut output, "== {} ==", name)?;
    //     writeln!(&mut output)?;

    //     writeln!(&mut output, "=== Constants ==")?;
    //     for (ix, constant) in self.consts.iter().enumerate() {
    //         writeln!(&mut output, "{:04} {:?}", ix, constant)?;
    //     }
    //     writeln!(&mut output)?;

    //     writeln!(&mut output, "=== Instructions ===")?;
    //     let mut ix = 0usize;

    //     while ix < self.code.len() {
    //         write!(&mut output, "{:04} ", ix)?;

    //         if ix > 0 && (self.node[ix] == self.node[ix - 1]) {
    //             write!(&mut output, "   | ")?;
    //         } else {
    //             write!(&mut output, "{:4} ", self.node[ix])?;
    //         }

    //         self.disassemble_instruction(&mut output, &mut ix)?;
    //     }

    //     writeln!(&mut output)?;
    //     writeln!(&mut output, "== end ==")?;

    //     Ok(output)
    // }

    pub fn disassemble_instruction(&self, output: &mut String, ix: &mut usize) -> std::fmt::Result {
        use std::fmt::Write;

        let opcode = match Opcode::try_from(self.code[*ix]) {
            Ok(opcode) => opcode,
            Err(unknown) => {
                return write!(output, "{:?}", unknown);
            }
        };

        write!(output, "{:?}", opcode)?;

        match opcode {
            | Opcode::Return
            | Opcode::Negate
            | Opcode::Add
            | Opcode::Sub
            | Opcode::Mult
            | Opcode::Divide
            | Opcode::Unit
            | Opcode::True
            | Opcode::False
            | Opcode::Pop
            | Opcode::Equal
            | Opcode::Greater
            | Opcode::Less
            | Opcode::Not
            | Opcode::Concatenate
            | Opcode::Print
            => {
                *ix += 1;
            },

            | Opcode::Constant
            | Opcode::PopNExceptTop
            => {
                if let Some(const_ix) = self.code.get(*ix + 1) {
                    if let Some(value) = self.consts.get(*const_ix as usize) {
                        write!(output, " {} {}", *const_ix, *value)?;
                    } else {
                        write!(output, " {} <Missing Value>", *const_ix)?;
                    }
                } else {
                    write!(output, " <EOF>")?;
                }

                *ix += 2;
            },

            #[allow(unreachable_patterns)]
            _ if false => {
                // This index cannot panic.
                match self.code[self.code.len().min(*ix + 1) .. self.code.len().min(*ix + 2)] {
                    [] => {
                        write!(output, " <Missing Value> <Missing Value>")
                    },

                    [rhs] => {
                        write!(output, " {:?}, <Missing Value>", rhs)
                    },

                    [rhs, lhs] => {
                        write!(output, " {:?}, {:?}", rhs, lhs)
                    },

                    _ => unreachable!()
                }?;

                *ix += 3;
            }
        }

        writeln!(output)
    }
}

#[derive(Debug)]
pub struct TooManyConstantsError;