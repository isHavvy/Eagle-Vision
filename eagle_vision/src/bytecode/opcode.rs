use crate::aspect::expression::MathOperator;

// WARNING: The ordering of the enum variants matters. See the TryFrom<u8> impl!
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Opcode {
    /// Push a constant value onto the stack
    Constant,

    /// Push the unit value constant onto the stack
    Unit,

    /// Push the true constant onto the stack
    True,

    /// Push the false constant onto the stack
    False,

    /// Remove the top value from the stack.
    Pop,

    /// Remove the top two values from the stack, push a boolean signifying if the two values are equal.
    Equal,

    /// Remove the top two values from the stack, push a boolean signifying if the second value is greater than the first.
    Greater,

    Less,
    Add,
    Sub,
    Mult,
    Divide,
    Not,
    Negate,
    Return,

    // #region non-compiler-book Not part of the compiler book.

    // Remove the top value from the stack and print it to the console.
    Print = 97,

    // Pops the top N values below the topmost value.
    PopNExceptTop = 98,

    // In the final language, this should be a built-in function.
    Concatenate = 99,
}

#[derive(Debug)]
pub struct UnknownOp;

impl TryFrom<u8> for Opcode {
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            00 => Opcode::Constant,
            01 => Opcode::Unit,
            02 => Opcode::True,
            03 => Opcode::False,
            04 => Opcode::Pop,
            05 => Opcode::Equal,
            06 => Opcode::Greater,
            07 => Opcode::Equal,
            08 => Opcode::Add,
            09 => Opcode::Sub,
            10 => Opcode::Mult,
            11 => Opcode::Divide,
            12 => Opcode::Not,
            13 => Opcode::Negate,
            14 => Opcode::Return,
            97 => Opcode::Print,
            98 => Opcode::PopNExceptTop,
            99 => Opcode::Concatenate,
            _ => Err(UnknownOp)?
        })
    }

    type Error = UnknownOp;
}

impl From<MathOperator> for Opcode {
    fn from(op: MathOperator) -> Self {
        match op {
            MathOperator::Plus => Opcode::Add,
            MathOperator::Minus => Opcode::Sub,
            MathOperator::Mult => Opcode::Mult,
            MathOperator::Divide => Opcode::Divide,
        }
    }
}