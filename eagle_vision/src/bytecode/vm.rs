use std::convert::TryFrom;

use crate::bytecode::{Opcode, Value};
use crate::bytecode::chunk::Chunk;

pub struct VirtualMachine {
    /// Instruction pointer
    ip: usize,
    stack: [Value; 256],
    stack_top: usize,
    chunk: Chunk,

    output_callback: Box<dyn FnMut(String)>
}

impl VirtualMachine {
    pub fn new(chunk: Chunk, callback: Box<dyn FnMut(String,)>) -> Self {
        const UNSET: Value = Value::Unset;

        Self {
            ip: 0,
            stack: [UNSET; 256],
            stack_top: 0,
            chunk,

            output_callback: callback
        }
    }

    pub fn interpret(&mut self) -> Result<Value, InterpretError> {
        #[cfg(feature = "vm-trace")]
        let mut string_buffer = String::with_capacity(256);

        #[cfg(feature = "vm-trace")]
        println!("Starting VM Trace");

        loop {
            #[cfg(feature = "vm-trace")] {
                println!("          ");
                for ix in 0..self.stack_top {
                    print!("[ {:?} ]", self.stack[ix])
                }
                println!();

                self.chunk.disassemble_instruction(&mut string_buffer, &mut self.ip.clone()).unwrap();
                println!("{}", string_buffer);
                string_buffer.clear();
            }

            let instruction = self.read_at_ip();

            match 
                Opcode::try_from(instruction)
                .map_err(|_e| InterpretError::InvalidBycode)?
            {
                Opcode::Unit => {
                    // Do Nothing
                },

                Opcode::Not => {
                    let temp = match self.stack_pop() {
                        Value::Bool(bool) => Value::Bool(!bool),
                        _ => Err(InterpretError::RuntimeError)?
                    };

                    self.stack_push(temp);
                }

                Opcode::Return => {
                    return Ok(self.stack_pop());
                },

                Opcode::Negate => {
                    let temp = match self.stack_pop() {
                        Value::I32(int) => Value::I32(-int),
                        _ => return Err(InterpretError::RuntimeError)
                    };

                    self.stack_push(temp);
                },

                Opcode::Add => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();

                    let add = match (a, b) {
                        (Value::I32(a), Value::I32(b)) => {
                            Value::I32(a.wrapping_add(b))
                        },

                        _ => Err(InterpretError::RuntimeError)?
                    };

                    self.stack_push(add);
                },

                Opcode::Sub => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();

                    let sub = match (a, b) {
                        (Value::I32(a), Value::I32(b)) => {
                            Value::I32(a.wrapping_sub(b))
                        },

                        _ => Err(InterpretError::RuntimeError)?
                    };

                    self.stack_push(sub);
                },

                Opcode::Mult => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();

                    let mult = match (a, b) {
                        (Value::I32(a), Value::I32(b)) => {
                            Value::I32(a.wrapping_mul(b))
                        },

                        _ => Err(InterpretError::RuntimeError)?
                    };

                    self.stack_push(mult);
                },

                Opcode::Divide => {
                    let b = self.stack_pop();
                    let a = self.stack_pop();

                    let div = match (a, b) {
                        (Value::I32(a), Value::I32(b)) => {
                            Value::I32(a.wrapping_div(b))
                        },

                        _ => Err(InterpretError::RuntimeError)?
                    };

                    self.stack_push(div);
                },

                Opcode::Constant => {
                    let const_ix = self.read_at_ip().into();
                    let constant = self.chunk.index_consts(const_ix);
                    self.stack_push(constant);
                },

                Opcode::True => {
                    self.stack_push(Value::Bool(true));
                },

                Opcode::False => {
                    self.stack_push(Value::Bool(false));
                },

                Opcode::Pop => {
                    let _ = self.stack_pop();
                }

                Opcode::Equal => {
                    let rhs = self.stack_pop();
                    let lhs = self.stack_pop();

                    let equal = match (lhs, rhs) {
                        (Value::Unset, _) | (_, Value::Unset) => panic!("Comparison with Unset performed"),
                        (Value::NoValue, Value::NoValue) => true,
                        (Value::NoValue, _) => return Err(InterpretError::RuntimeError),
                        (Value::I32(lhs), Value::I32(rhs)) => lhs == rhs,
                        (Value::I32(..), _) => Err(InterpretError::RuntimeError)?,
                        (Value::F64(lhs), Value::F64(rhs)) => lhs == rhs,
                        (Value::F64(..), _) => Err(InterpretError::RuntimeError)?,
                        (Value::Bool(lhs), Value::Bool(rhs)) => lhs == rhs,
                        (Value::Bool(..), _) => Err(InterpretError::RuntimeError)?,
                        // (Value::Object(_lhs), Value::Object(_rhs)) => unimplemented!(),
                        // (Value::Object(_), _) => return Err(InterpretError::RuntimeError),
                        (Value::String(lhs), Value::String(rhs)) => lhs == rhs,
                        (Value::String(..), _) => Err(InterpretError::RuntimeError)?,
                    };

                    self.stack_push(Value::Bool(equal));
                },

                Opcode::Greater => {
                    let rhs = self.stack_pop();
                    let lhs = self.stack_pop();

                    #[allow(clippy::bool_comparison)]
                    let greater = match (lhs, rhs) {
                        (Value::Unset, _) | (_, Value::Unset) => panic!("Comparison with Unset performed"),
                        (Value::NoValue, Value::NoValue) => true,
                        (Value::NoValue, _) => return Err(InterpretError::RuntimeError),
                        (Value::I32(lhs), Value::I32(rhs)) => lhs > rhs,
                        (Value::I32(..), _) => Err(InterpretError::RuntimeError)?,
                        (Value::F64(lhs), Value::F64(rhs)) => lhs > rhs,
                        (Value::F64(..), _) => Err(InterpretError::RuntimeError)?,
                        (Value::Bool(lhs), Value::Bool(rhs)) => lhs > rhs,
                        (Value::Bool(..), _) => Err(InterpretError::RuntimeError)?,
                        // (Value::Object(_lhs), Value::Object(_rhs)) => unimplemented!(),
                        // (Value::Object(_), _) => return Err(InterpretError::RuntimeError),
                        (Value::String(..), _) => Err(InterpretError::RuntimeError)?,
                    };

                    self.stack_push(Value::Bool(greater));
                },

                Opcode::Less => {
                    let rhs = self.stack_pop();
                    let lhs = self.stack_pop();

                    #[allow(clippy::bool_comparison)]
                    let less = match (lhs, rhs) {
                        (Value::Unset, _) | (_, Value::Unset) => panic!("Comparison with Unset performed"),
                        (Value::NoValue, Value::NoValue) => true,
                        (Value::NoValue, _) => return Err(InterpretError::RuntimeError),
                        (Value::I32(lhs), Value::I32(rhs)) => lhs < rhs,
                        (Value::I32(..), _) => Err(InterpretError::RuntimeError)?,
                        (Value::F64(lhs), Value::F64(rhs)) => lhs < rhs,
                        (Value::F64(..), _) => Err(InterpretError::RuntimeError)?,
                        (Value::Bool(lhs), Value::Bool(rhs)) => lhs < rhs,
                        (Value::Bool(..), _) => Err(InterpretError::RuntimeError)?,
                        // (Value::Object(_lhs), Value::Object(_rhs)) => unimplemented!(),
                        // (Value::Object(_), _) => return Err(InterpretError::RuntimeError),
                        (Value::String(..), _) => Err(InterpretError::RuntimeError)?,
                    };

                    self.stack_push(Value::Bool(less));
                }

                Opcode::Print => {
                    let string = self.stack_pop();

                    match string {
                        // Value::Object(object) => {
                        //     match object.body {
                        //         ObjectContents::StringBuf(ref string) => {
                        //             (self.output_callback)(string.clone())
                        //         },
                        //     }
                        // },

                        Value::I32(int) => {
                            (self.output_callback)(format!("{}\n", int.to_string()))
                        }

                        _ => Err(InterpretError::RuntimeError)?
                    }

                    self.stack_push(Value::NoValue)
                }

                Opcode::Concatenate => {
                    let rhs = self.stack_pop();
                    let lhs = self.stack_pop();

                    let (lhs, rhs) = match (lhs, rhs) {
                        (Value::String(lhs), Value::String(rhs)) => (lhs, rhs),
                        _ => Err(InterpretError::RuntimeError)?
                    };

                    let concatenated = Value::String(format!("{lhs}{rhs}"));
                    self.stack_push(concatenated);
                }

                Opcode::PopNExceptTop => {
                    let n = self.read_at_ip();
                    self.stack_pop_n_except_top(n);
                }
            }
        }
    }

    fn read_at_ip(&mut self) -> u8 {
        let res = self.chunk.index_code(self.ip);
        self.ip += 1;
        res
    }

    fn stack_push(&mut self, value: Value) {
        self.stack[self.stack_top] = value;
        self.stack_top += 1;
    }

    fn stack_pop(&mut self) -> Value {
        self.stack_top -= 1;
        self.stack[self.stack_top].take()
    }

    fn stack_pop_n_except_top(&mut self, n: u8) {
        let n = n as usize;
        let top = self.stack[self.stack_top].take();
        self.stack[self.stack_top - n] = top;
        self.stack_top -= n;
    }

    pub fn capture_output(&mut self, callback: Box<dyn FnMut(String,)>) {
        self.output_callback = callback;
    }
}

#[derive(Debug)]
pub enum InterpretError {
    InvalidBycode,
    RuntimeError,
}

#[cfg(test)]
mod test {
    use bevy::prelude::*;
    use crate::bytecode::{Opcode, Value, chunk::Chunk};
    use super::VirtualMachine;

    #[test]
    fn concat() {
        // 'se' meaning 'source entity'
        let [se0, se1, se2, se3] = [0, 1, 2, 3].map(|n| Entity::from_raw_u32(n).unwrap());

        let mut chunk = Chunk::new();
        // UNWRAP: These unwraps are safe because only 2 constants are being placed in this chunk.
        let rhs = chunk.push_const(Value::String("xyz".to_owned())).unwrap();
        let lhs = chunk.push_const(Value::String("abc".to_owned())).unwrap();
        chunk.push_opcode(Opcode::Constant, se0);
        chunk.push_code(lhs, se0);
        chunk.push_opcode(Opcode::Constant, se1);
        chunk.push_code(rhs, se1);
        chunk.push_opcode(crate::bytecode::Opcode::Concatenate, se2);
        chunk.push_opcode(Opcode::Return, se3);

        // println!("{}", chunk.disassemble("concat example".into()).unwrap());

        let mut vm = VirtualMachine::new(chunk, Box::new(|_| ()));
        let res = vm.interpret();

        match res.unwrap() {
            Value::String(return_string) => {
                assert_eq!(&return_string, "abcxyz");
            },

            _ => assert!(false)
        }
    }
}