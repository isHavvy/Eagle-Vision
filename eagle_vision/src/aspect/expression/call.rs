use bevy::prelude::*;

use crate::eaglevision::{EagleVision, GuaranteedMarkerComponent};
use crate::aspect::ty::{ActualTy, TyUnknown};
use crate::aspect::expression::Expression;

use super::ExpressionKind;

#[derive(Debug, Component)]
pub struct CallExpression {
    pub callable: Entity,
    pub arguments: Vec<Entity>,
}

impl CallExpression {
    pub fn spawn(ev: &mut EagleVision, call_expression: CallExpression) -> Entity {
        todo!()
    //     let unknown = TyUnknown::get(ev);

    //     ev.spawn((
    //         Expression {
    //             kind: ExpressionKind::Call,
    //             expected_ty: None,
    //         },

    //         call_expression,

    //         // #[TODO(Havvy)]: Figure out the actual ty.
    //         ActualTy(unknown),
    //     )).id()
    }
}