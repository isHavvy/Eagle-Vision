use bevy::prelude::*;

use crate::{eaglevision::{EagleVision, GuaranteedMarkerComponent}, aspect::{expression::{Expression, ExpressionKind}, ty::{ActualTy, TyUnknown}}};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum BindingExpression {
    /// Refers to a parameter in the function containing this expression.
    /// The Entity is a reference to the function.
    Parameter(usize, Entity)
}

impl BindingExpression {
    pub fn spawn(
        ev: &mut EagleVision,
        variable_expression: BindingExpression,
    ) -> Entity {
        todo!();
        // match variable_expression {
        //     BindingExpression::Parameter(ix, function) => {
        //         let ty = match ev.function(function).get_parameter(ix) {
        //             Some(parameter) => ev.function_parameter(parameter).ty(),
        //             None => TyUnknown::get(ev),
        //         };

        //         ev.spawn((
        //             variable_expression,

        //             Expression {
        //                 kind: ExpressionKind::Binding,
        //                 expected_ty: None,
        //             },

        //             ActualTy(ty)
        //         )).id()
        //     },
        // }
    }
}