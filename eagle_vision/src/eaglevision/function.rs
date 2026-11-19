use bevy::prelude::*;
use crate::aspect::{expression::{Expression, ExpressionKind, SubexpressionOf}, function::Function};

use super::EagleVision;

/// A mutable reference to a specific function and the entire EagleVision source.
pub struct EagleVisionFunctionMut<'ev>(EntityWorldMut<'ev>);

impl<'ev> EagleVisionFunctionMut<'ev> {
    pub(super) fn new(ev: &'ev mut EagleVision, function: Entity) -> Self {
        Self(ev.app.world_mut().entity_mut(function))
    }

    fn get_function_component_mut(&mut self) -> Mut<'_, Function> {
        self.0.get_mut::<Function>().unwrap()
    }

    pub fn push_parameter(&mut self, parameter: Entity) {
        self.get_function_component_mut().push_parameter(parameter);
    }

    pub fn set_return_ty(&mut self, ty: Entity) {
        self.get_function_component_mut().return_ty = ty;
    }

    /// Replace the body expression of the function with a new one.
    /// 
    /// If the expression was a Missing expression, it gets deleted.
    /// Otherwise, the Entity referring to the old expression is returned.
    /// It is a free floating expression.
    /// Also, it might fail to type check because it is no longer being
    /// coerced to the return type of the function.
    pub fn replace_body(&mut self, new_body: Entity) -> Option<Entity> {
        let old_body = std::mem::replace(&mut self.get_function_component_mut().body, new_body);

        self.0.world_scope(|world| -> Option<Entity> {
            let mut old_body_mut = world.entity_mut(old_body);
            let mut expr = old_body_mut.get_mut::<Expression>().unwrap();

            if expr.kind == ExpressionKind::Missing {
                world.despawn(old_body);
                None
            } else {
                expr.expected_ty = None;
                old_body_mut.remove::<SubexpressionOf>();
                Some(old_body)
            }
        })
    }
}

/// A reference to a specific function and the entire EagleVision source.
pub struct EagleVisionFunction<'ev>(EntityRef<'ev>);

impl<'ev> EagleVisionFunction<'ev> {
    pub(super) fn new(ev: &'ev EagleVision, function: Entity) -> Self {
        Self(ev.app.world().entity(function))
    }

    fn get_function_component(&self) -> &Function {
        self.0.get::<Function>().unwrap()
    }

    pub fn get_parameter(&self, ix: usize) -> Option<Entity> {
        let function = self.get_function_component();
        function.parameters.get(ix).copied()
    }
}