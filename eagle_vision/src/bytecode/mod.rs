use bevy::prelude::*;

use crate::bytecode::opcode::Opcode;

pub mod chunk;
pub mod compile;
pub mod opcode;
pub mod program;
pub mod vm;

#[derive(Debug, Clone)]
pub enum Value {
    Unset,
    NoValue,
    I32(i32),
    F64(f64),
    String(String),
    Bool(bool),
}

impl Value {
    pub fn take(&mut self) -> Self {
        std::mem::replace(self, Value::Unset)
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unset => f.write_str("<< Unset >>"),
            Value::NoValue => f.write_str("<< No Value >>"),
            Value::I32(value) => write!(f, "{}", value),
            Value::F64(value) => write!(f, "{}", value),
            Value::String(value) => write!(f, "{}", value),
            Value::Bool(value) => write!(f, "{}", value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
struct CompiledChunk(Entity);

impl ContainsEntity for CompiledChunk {
    fn entity(&self) -> Entity {
        self.0
    }
}