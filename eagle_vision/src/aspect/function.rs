use bevy::prelude::*;

use crate::aspect::ty::{ActualTy, TyUnknown};
use crate::aspect::{ty::TyUnit};
use crate::aspect::text::Text;
use crate::aspect::expression::{Expression, ExpressionKind, SubexpressionOf};

#[derive(Debug, Component)]
pub struct Function {
    pub(crate) body: Entity, // Expression
    pub(crate) parameters: Vec<Entity>, // Ty + Text?
    pub(crate) return_ty: Entity // Ty
}

impl Function {
    pub(crate) fn spawn(

        // The Entity must have the Locale component.
        name: Option<(Entity, impl Into<String>)>,
    ) -> Entity {
        todo!()
        // let unit = TyUnit::get(ev);
        // let unknown = TyUnknown::get(ev);
        // let function = ev.spawn_empty_id();

        // let body = ev.spawn_id((
        //     Expression {
        //         kind: ExpressionKind::Missing,
        //         expected_ty: Some(unit),
        //     },
        //     ActualTy(unknown),
        //     SubexpressionOf(function),
        // ));

        // // let text = if let Some((locale, name)) = name {
        // //     Text::new(locale, name)
        // // } else {
        // //     Text::new_empty()
        // // };
        // let text = Text::new_empty();

        // ev.insert_into(function, (
        //     text,
        //     Function {
        //         body,
        //         parameters: vec![],
        //         return_ty: unit
        //     }
        // ));

        // function
    }

    pub(crate) fn push_parameter(&mut self, parameter: Entity) {
        self.parameters.push(parameter);
    }
}