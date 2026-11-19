//! Applies whatever the graph can represent; the diagnostics set
//! after ExecuteOps judges validity. An unresolved TagId is inserted
//! anyway (dangling refs are representable, like ExpressionKind::Missing,
//! and may resolve later in the batch). An unknown *target* is a no-op:
//! no entity means nowhere to put the component.
use crate::bevy::prelude::*;

use eagle_squak::id::{AttributeDefinitionId, AttributeId, EagleVisionId, EditorId};

use crate::aspect::editor::LastEditedBy;
use super::{OpCompleted, OpId, op_item};

op_item!(CreateAttribute, CreateAttributeUnvalidated {
    op: valid OpId,
    id: new AttributeId,
    definition: existing AttributeDefinitionId,
    on: existing EagleVisionId,
    editor: valid EditorId,
});

pub fn on_apply_tag(
    event: On<CreateAttribute>,
    mut commands: Commands,
) {
    commands.spawn_attribute(event.id, event.definition, event.on).insert(LastEditedBy(event.editor));
    commands.trigger(OpCompleted { op: event.op });
}