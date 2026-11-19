use bevy::prelude::*;

use crate::bytecode::{chunk::TooManyConstantsError, CompiledChunk, Value};
use crate::eaglevision::{EagleVision};
use crate::bytecode::{chunk::Chunk, opcode::Opcode};
use crate::aspect::expression::{self, *};

// fn compile_statement(expressions: &Expressions, statement: &Statement, chunk: &mut Chunk, compiler: &mut Compiler) {
//     const FAKE_NODE: u32 = 42069;

//     match statement {
//         Statement::Expression(index) => {
//             compile_expr(expressions, *index, chunk, compiler);
//             chunk.push_opcode(Opcode::Pop, FAKE_NODE + 2);
//         }
//         Statement::Declare(_) => { todo!(); }
//         Statement::Print(index) => {
//             compile_expr(expressions, *index, chunk, compiler);
//             chunk.push_opcode(Opcode::Print, FAKE_NODE + 1);
//         }
//     }
// }


// Expression::Block(locals, statements, expr) => {
//     compiler.create_scope();
    
//     for _local in locals {
//         chunk.push_opcode(Opcode::Unit, index.inner());
//     }
    
//     for statement in statements {
//         compile_statement(expressions, statement, chunk, compiler);
//     }
//     compile_expr(expressions, *expr, chunk, compiler);
//     chunk.push_opcode(Opcode::PopNExceptTop, index.inner());
//     chunk.push_code(locals.len() as u8, index.inner());
//     compiler.end_scope();
// }
// Expression::ComparisonOp(comparison_op, lhs, rhs) => {
//     compile_expr(expressions, *lhs, chunk, compiler);
//     compile_expr(expressions, *rhs, chunk, compiler);
//     chunk.push_opcode(Opcode::from(*comparison_op), index.inner());
// },
// Expression::Concatenate(lhs, rhs) => {
//     compile_expr(expressions, *lhs, chunk, compiler);
//     compile_expr(expressions, *rhs, chunk, compiler);
//     chunk.push_opcode(Opcode::Concatenate, index.inner());
// },

#[derive(Debug)]
pub enum CompileError {
    MissingExpression(Entity),
    /// A chunk has a maximum of 256 constants it can have it.
    TooManyConstants,
    UnspecifiedError,
}

impl From<TooManyConstantsError> for CompileError {
    fn from(_: TooManyConstantsError) -> Self {
        CompileError::TooManyConstants
    }
}

pub(crate) fn compile_executable<'ev>(mut ev: EagleVision, /* executable: &mut EagleVisionExecutable<'ev, MutableStorage<'ev>> */) -> Result<Entity, CompileError> {
    todo!()
    
    // let mut chunk = Chunk::new();
    // // let main_block = executable.main_block();
    // // let main_block_ctx = executable.source().expression(main_block);
    // // let main_block_ctx = main_block_ctx.as_kind_block();
    // // let statements = main_block_ctx.statements();

    // // for expression in statements.iter().copied() {
    // //     compile_expression(ev, &mut chunk, expression)?;
    // // }
    // // chunk.push_opcode(Opcode::Return, main_block);

    // let chunk = ev.spawn_id(chunk);
    // // ev.insert_into(executable, CompiledChunk(chunk));
    // Ok(chunk)
}

type EagleVisionExecutable<'ev> = &'ev ();

fn compile_exectuable<'ev>(exe_ctx: EagleVisionExecutable<'ev>) -> Result<Entity, CompileError> {
    // let mut program = Program::new();
    // let chunk = Chunk::new();

    // let main_block = exe_ctx.main_block();
    // let (lang, exe_entity) = exe_ctx.finish();
    // let main_block_expr = lang.expression(main_block);
    // let statements = main_block_expr.statements();

    // for expression in statements.iter().copied() {
    //     compile_expression(main_block_expr.lang(), &mut chunk, expression)?;
    // }
    // chunk.push_opcode(Opcode::Return);
    // program.insert_main_chunk(chunk);

    // let program = lang.world.spawn(Program(program)).id();
    // lang.executable_mut().set_compilation(program);
    todo!()
}

fn compile_expression(ev: &EagleVision, chunk: &mut Chunk, expression_entity: Entity) -> Result<(), CompileError> {
    todo!()
    // let expression_ctx = ev.expression(expression_entity);
    // let expression = expression_ctx.get_expression_component();

    // match expression.kind {
    //     ExpressionKind::Missing => Err(CompileError::MissingExpression(expression_entity)),
    //     ExpressionKind::Block => Err(CompileError::UnspecifiedError),
    //     ExpressionKind::Call => Err(CompileError::UnspecifiedError),
    //     ExpressionKind::Binding => Err(CompileError::UnspecifiedError),
    //     ExpressionKind::CoreLiteral => {
    //         let literal = expression_ctx.as_kind_literal().value();

    //         match literal {
    //             CoreLiteralValue::Unit => {},
    //             CoreLiteralValue::Int32(value) => {
    //                 let const_ix = chunk.push_const(Value::I32(value.clone()))?;
    //                 chunk.push_opcode(Opcode::Constant, expression_entity);
    //                 chunk.push_code(const_ix, expression_entity);
    //             },
    //             CoreLiteralValue::Float64(value) => {
    //                 let const_ix = chunk.push_const(Value::F64(value.clone()))?;
    //                 chunk.push_opcode(Opcode::Constant, expression_entity);
    //                 chunk.push_code(const_ix, expression_entity);
    //             },
    //             CoreLiteralValue::Boolean(true) => {
    //                 chunk.push_opcode(Opcode::True, expression_entity);
    //             },
    //             CoreLiteralValue::Boolean(false) => {
    //                 chunk.push_opcode(Opcode::False, expression_entity);
    //             }
    //             CoreLiteralValue::Utf8(value) => {
    //                 let const_ix = chunk.push_const(Value::String(value.clone()))?;
    //                 chunk.push_opcode(Opcode::Constant, expression_entity);
    //                 chunk.push_code(const_ix, expression_entity);
    //             },
    //         }

    //         Ok(())
    //     },
    //     ExpressionKind::MathOperator => {
    //         let MathOperatorExpression { op, lhs, rhs } = expression_ctx.as_kind_math_operation().get_math_operator_expression();

    //         compile_expression(ev, chunk, lhs)?;
    //         compile_expression(ev, chunk, rhs)?;

    //         chunk.push_opcode(Opcode::from(op), expression_entity);

    //         Ok(())
    //     },
    //     ExpressionKind::ComparisonOperator => Err(CompileError::UnspecifiedError),
    //     ExpressionKind::Print => {
    //         let inner_expression = expression_ctx.as_unary().inner_expression();
    //         compile_expression(ev, chunk, inner_expression)?;
    //         chunk.push_opcode(Opcode::Print, expression_entity);
    //         Ok(())
    //     },
    // }
}