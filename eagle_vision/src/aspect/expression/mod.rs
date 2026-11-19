use bevy::prelude::*;

use crate::{eaglevision::{EagleVision, GuaranteedMarkerComponent}, aspect::ty::{ActualTy, TyUnit}};

#[derive(Debug, PartialEq, Eq)]
pub enum ExpressionKind {
    /// Spot where an expression should go, but there is no actual expression.
    Missing,

    /// A sequence of expressions that evaluates to its final expression.
    Block,

    /// Accessing a binding (parameter or local)
    Binding,

    /// An expression of a core ty that evaluates to a constant declared in the source. 
    CoreLiteral,

    /// Algebraic operations.
    MathOperator,

    /// Ordered operations.
    ComparisonOperator,

    /// Calling functions
    Call,

    /// Prints the value to `stdout`.
    Print
}

#[derive(Debug, Component)]
pub struct Expression {
    pub(crate) kind: ExpressionKind,
    pub(crate) expected_ty: Option<Entity>,
}

#[derive(Debug, Component)]
pub struct SubexpressionOf(pub Entity);

impl ContainsEntity for SubexpressionOf {
    fn entity(&self) -> Entity {
        self.0
    }
}

#[derive(Debug, Component)]
pub struct UnaryExpression(Entity);

impl ContainsEntity for UnaryExpression {
    fn entity(&self) -> Entity {
        self.0
    }
}

pub(crate) fn spawn_print_expression(ev: &mut EagleVision, inner_expression: Entity) -> Entity {
    todo!()
    // let unit = TyUnit::get(ev);
    // let print_expression = ev.spawn_id((
    //     Expression {
    //         kind: ExpressionKind::Print,
    //         expected_ty: None,
    //     },
    //     UnaryExpression(inner_expression),
    //     ActualTy(unit),
    // ));

    // ev.expression_mut(inner_expression).set_subexpression_of(print_expression);

    // print_expression
}

// Individual expression kind data

mod binding;
pub use binding::BindingExpression;

mod block;
pub use block::BlockExpression;
pub(crate) use block::push_block_expression_statement;

mod call;
pub use call::CallExpression;

mod literal;
pub use literal::{LiteralExpression, CoreLiteralValue};

mod math;
pub use math::{MathOperator, MathOperatorExpression};