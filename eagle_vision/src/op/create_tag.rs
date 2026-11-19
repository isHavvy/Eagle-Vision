use crate::bevy::prelude::*;
use eagle_squak::id::{EditorId, AttributeDefinitionId};

use crate::aspect::editor::LastEditedBy;
use super::{OpCompleted, OpId, op_item};

op_item! { CreateAttributeDefinition, CreateAttributeDefinitionUnvalidated {
    op: valid OpId,
    editor: valid EditorId,
    id: new AttributeDefinitionId,
}}

pub fn on_create_attribute_definition(
    event: On<CreateAttributeDefinition>,
    mut commands: Commands,
) {
    commands.spawn_attribute_definition(event.id).insert((
        LastEditedBy(event.editor),
    ));
    commands.trigger(OpCompleted { op: event.op });
}