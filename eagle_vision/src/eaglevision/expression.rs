use bevy::prelude::*;

use crate::aspect::ty::ActualTy;
use crate::aspect::expression::*;
use crate::eaglevision::EagleVision;

/// A mutable reference to a specific expression and the entire EagleVision source.
pub struct EagleVisionExpressionMut<'ev>(EntityWorldMut<'ev>);

impl<'ev> EagleVisionExpressionMut<'ev> {
    pub(super) fn new(ev: &'ev mut EagleVision, expression: Entity) -> Self {
        Self(ev.app.world_mut().entity_mut(expression))
    }

    pub fn set_subexpression_of(&mut self, parent: Entity) {
        self.0.insert(SubexpressionOf(parent));
    }
}

/// A reference to a specific expression and the entire EagleVision source.
pub struct EagleVisionExpression<'ev>(EntityRef<'ev>);

impl<'ev> EagleVisionExpression<'ev> {
    pub(super) fn new(ev: &'ev EagleVision, expression: Entity) -> Self {
        Self(ev.app.world().entity(expression))
    }

    pub(crate) fn get_expression_component(&self) -> &Expression {
        self.0.get::<Expression>().unwrap()
    }

    pub fn actual_ty(&self) -> Entity {
        self.0.get::<ActualTy>().unwrap().entity()
    }

    pub(crate) fn as_kind_block(&self) -> EagleVisionBlockExpression {
        EagleVisionBlockExpression(self.0)
    }

    pub(crate) fn as_kind_literal(&self) -> EagleVisionLiteralExpression {
        EagleVisionLiteralExpression(self.0)
    }

    pub(crate) fn as_kind_math_operation(&self) -> EagleVisionMathOperationExpression {
        EagleVisionMathOperationExpression(self.0)
    }

    pub(crate) fn as_unary(&self) -> EagleVisionUnaryExpression {
        EagleVisionUnaryExpression(self.0)
    }
}

/// A reference to a specific block expression and the entire EagleVision source.
pub struct EagleVisionBlockExpression<'ev>(EntityRef<'ev>);

impl<'ev> EagleVisionBlockExpression<'ev> {
    pub fn statements(&self) -> &[Entity] {
        &self.0.get::<BlockExpression>().unwrap().statements[..]
    }
}

/// A reference to a specific literal expression and the entire EagleVision source.
pub struct EagleVisionLiteralExpression<'ev>(EntityRef<'ev>);

impl<'ev> EagleVisionLiteralExpression<'ev> {
    pub fn value(&self) -> CoreLiteralValue {
        self.0.get::<LiteralExpression>().unwrap().value.clone()
    }
}

/// A reference to a specific math operation expression and the entire EagleVision source.
pub struct EagleVisionMathOperationExpression<'ev>(EntityRef<'ev>);

impl<'ev> EagleVisionMathOperationExpression<'ev> {
    pub(crate) fn get_math_operator_expression(&self) -> MathOperatorExpression {
        self.0.get::<MathOperatorExpression>().unwrap().clone()
    }
}

/// A reference to a specific unary expression and the entire EagleVision source.
pub struct EagleVisionUnaryExpression<'ev>(EntityRef<'ev>);

impl<'ev> EagleVisionUnaryExpression<'ev> {
    pub(crate) fn inner_expression(&self) -> Entity {
        self.0.get::<UnaryExpression>().unwrap().entity()
    }
}