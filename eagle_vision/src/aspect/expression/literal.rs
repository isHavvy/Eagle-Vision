use bevy::prelude::*;
use super::{Expression, ExpressionKind};

#[derive(Debug, Component)]
pub struct LiteralExpression {
    pub(crate) value: CoreLiteralValue,
}

impl LiteralExpression {
    pub(crate) fn spawn(value: CoreLiteralValue) -> Entity {
        todo!();
        // let actual_ty = ActualTy(value.ty(ev));

        // ev.spawn_id((
        //     Expression {
        //         kind: ExpressionKind::CoreLiteral,
        //         expected_ty: None,
        //     },

        //     LiteralExpression { value, },

        //     actual_ty,
        // ))
    }
}

#[derive(Debug, Clone)]
pub enum CoreLiteralValue {
    Unit,
    Int32(i32),
    Float64(f64),
    Boolean(bool),
    Utf8(String),
}

impl CoreLiteralValue {
    pub fn ty(&self) -> Entity {
        todo!()
        // match self {
        //     CoreLiteralValue::Unit => TyUnit::get(ev),
        //     CoreLiteralValue::Int32(_) => TyInt32::get(ev),
        //     CoreLiteralValue::Float64(_) => TyFloat64::get(ev),
        //     CoreLiteralValue::Boolean(_) => TyBoolean::get(ev),
        //     CoreLiteralValue::Utf8(_) => TyUtf8::get(ev),
        // }
    }
}