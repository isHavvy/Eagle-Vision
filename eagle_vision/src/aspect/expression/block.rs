use bevy::prelude::*;

use crate::aspect::expression::SubexpressionOf;

#[derive(Debug, Component)]
pub struct BlockExpression {
    pub(crate) statements: Vec<Entity>,
}

impl BlockExpression {
    pub fn new() -> Self {
        Self {
            statements: vec![],
        }
    }
}

pub(crate) fn push_block_expression_statement(world: &mut World, block_expression: Entity, statement_expression: Entity) {
    world.entity_mut(statement_expression).insert(SubexpressionOf(block_expression));

    let mut block = world.entity_mut(block_expression);
    let mut block = block.get_mut::<BlockExpression>().unwrap();
    block.statements.push(statement_expression);
}