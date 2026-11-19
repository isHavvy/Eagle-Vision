use bevy::prelude::*;

use crate::aspect::expression::Expression;
use crate::aspect::ty::{ActualTy, TyInt32, TyUnknown};
use crate::eaglevision::{EagleVision, GuaranteedMarkerComponent};

use super::ExpressionKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathOperator {
    Plus,
    Minus,
    Mult,
    Divide,
}

#[derive(Debug, Clone, Copy, Component)]
pub struct MathOperatorExpression {
    pub(crate) op: MathOperator,
    pub(crate) lhs: Entity,
    pub(crate) rhs: Entity,
}

impl MathOperatorExpression {
    pub(crate) fn spawn(
        ev: &mut EagleVision,
        op: MathOperator,
        lhs: Entity,
        rhs: Entity,
        expected_ty: Option<Entity>,
    ) -> Entity {
        todo!()
        // let lhs_ty = ev.expression(lhs).actual_ty();
        // let rhs_ty = ev.expression(rhs).actual_ty();

        // let ty_i32 = TyInt32::get(ev);
        // let ty_unknown = TyUnknown::get(ev);

        // let actual_ty = ActualTy(if lhs_ty == ty_i32 && rhs_ty == ty_i32 {
        //     ty_i32
        // } else {
        //     ty_unknown
        // });

        // ev.spawn_id((
        //     Expression {
        //         kind: ExpressionKind::MathOperator,
        //         expected_ty,
        //     },

        //     MathOperatorExpression {
        //         op,
        //         lhs,
        //         rhs,
        //     },

        //     actual_ty,
        // ))
    }
}