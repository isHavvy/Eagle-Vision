use eagle_squak::id::{EagleVisionId, CommentId, EditorId, TextId};

use crate::aspect::editor::LastEditedBy;
use crate::bevy::prelude::*;
use super::{OpCompleted, OpId, op_item};

op_item!(CreateComment, CreateCommentUnvalidated {
    op: valid OpId,
    comment: new CommentId,
    body: existing TextId,
    on: existing EagleVisionId,
    editor: valid EditorId,
});

pub fn on_create_comment(
    event: On<CreateComment>,
    mut commands: Commands,
) {
    commands.spawn_comment(event.comment, event.body, event.on).insert(LastEditedBy(event.editor));
    commands.trigger(OpCompleted { op: event.op });
}

#[cfg(test)]
mod tests {
    use super::*;
    use eagle_squak::aspect::AspectId;
    use eagle_squak::id::{Existing, New};
    use crate::aspect::comment::{Comment, CommentOn};
    use crate::engine_plugin::IdEntityMap;
    use crate::op::{OpRejectionReason, UnvalidatedOp};
    use crate::op::test_util::*;

    fn unvalidated(op: u64, comment: CommentId, body: TextId, on: EagleVisionId) -> CreateCommentUnvalidated {
        CreateCommentUnvalidated {
            op: OpId(op),
            comment: New::claim(comment),
            body: Existing::claim(body),
            on: Existing::claim(on),
            editor: EDITOR,
        }
    }

    /// Any aspect can be a comment target. A tag stands in for a function
    /// until functions have a real spawn path.
    fn spawn_target(world: &mut World) -> EagleVisionId {
        let id = EagleVisionId::new(CLIENT.get(), AspectId::Function, 0);
        spawn_mapped(world, id, ());
        id
    }

    #[test]
    fn check_passes_when_body_and_target_exist() {
        let mut world = world_with_map();
        let c = client();
        let body = c.next_text();
        let comment = c.next_comment();
        spawn_text(&mut world, body, "because the parser requires it");
        let on = spawn_target(&mut world);

        let checked = unvalidated(0, comment, body, on)
            .check(world.resource::<IdEntityMap>(), CLIENT)
            .unwrap();

        assert_eq!(checked.comment, comment);
        assert_eq!(checked.body, body);
        assert_eq!(checked.on, on);
    }

    #[test]
    fn check_rejects_missing_body() {
        let mut world = world_with_map();
        let c = client();
        let body = c.next_text(); // missing
        let comment = c.next_comment();
        let on = spawn_target(&mut world);

        let err = unvalidated(0, comment, body, on)
            .check(world.resource::<IdEntityMap>(), CLIENT)
            .unwrap_err();

        assert_eq!(err, OpRejectionReason::IdNotFound(body.as_untyped()));
    }

    #[test]
    fn check_rejects_missing_target() {
        let mut world = world_with_map();
        let c = client();
        let body = c.next_text();
        let comment = c.next_comment();
        spawn_text(&mut world, body, "why");
        let on = ghost(CLIENT, AspectId::Function);

        let err = unvalidated(0, comment, body, on)
            .check(world.resource::<IdEntityMap>(), CLIENT)
            .unwrap_err();

        assert_eq!(err, OpRejectionReason::IdNotFound(on));
    }

    #[test]
    fn check_rejects_duplicate_comment_id() {
        let mut world = world_with_map();
        let c = client();
        let body = c.next_text();
        let comment = c.next_comment();
        spawn_text(&mut world, body, "why");
        let on = spawn_target(&mut world);
        spawn_mapped(&mut world, comment, (Comment { body }, CommentOn(on)));

        let err = unvalidated(0, comment, body, on)
            .check(world.resource::<IdEntityMap>(), CLIENT)
            .unwrap_err();

        assert_eq!(err, OpRejectionReason::IdAlreadyExists(comment.as_untyped()));
    }

    #[test]
    fn check_accepts_foreign_owned_references() {
        // `existing` checks presence, not ownership: commenting on core
        // library data is legitimate.
        let mut world = world_with_map();
        let c = client();
        let comment = c.next_comment();
        let core_body = EagleVisionId::new(CORE.get(), AspectId::Text, 0);
        let core_body = TextId::from_untyped(core_body).unwrap();
        spawn_text(&mut world, core_body, "core doc text");
        let on = EagleVisionId::new(CORE.get(), AspectId::Function, 0);
        spawn_mapped(&mut world, on, ());

        let checked = unvalidated(0, comment, core_body, on)
            .check(world.resource::<IdEntityMap>(), CLIENT)
            .unwrap();

        assert_eq!(checked.body, core_body);
        assert_eq!(checked.on, on);
    }

    #[test]
    fn observer_spawns_comment_with_attachment() {
        let mut world = world_with_map();
        world.add_observer(on_create_comment);
        let c = client();
        let body = c.next_text();
        let comment = c.next_comment();
        spawn_text(&mut world, body, "why");
        let on = spawn_target(&mut world);

        world.trigger(CreateComment { op: OpId(0), comment, body, on, editor: EDITOR });
        world.flush();

        let mut q = world.query::<(Entity, &Comment, &CommentOn, &LastEditedBy)>();
        let (entity, data, attached, edited) = q.single(&world).unwrap();
        assert_eq!(data.body, body);
        assert_eq!(attached.0, on);
        assert_eq!(edited.0, EDITOR);
        assert_eq!(world.resource::<IdEntityMap>().get(comment), Some(entity));
    }
}