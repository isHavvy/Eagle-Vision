use bevy::ecs::query::QuerySingleError;
use eagle_squak::id::{EditorId, LocaleId};

use crate::aspect::editor::LastEditedBy;
use crate::aspect::text::Text;
use crate::aspect::metadata::ProjectName;
use crate::bevy::prelude::*;
use crate::engine_plugin::EngineIdGen;
use crate::op::{OpCompleted, OpId};

#[derive(Debug, Event)]
pub struct SetProjectName {
    pub op: OpId,
    pub name: String,
    pub editor: EditorId,
    pub locale: LocaleId,
}

pub fn on_set_project_name(
    event: On<SetProjectName>,
    mut commands: Commands,
    mut project_name_query: BevyQuery<(Entity, &mut Text), With<ProjectName>>,
    ids: Res<EngineIdGen>,
) {
    match project_name_query.single_mut() {
        Ok((eid, mut text)) => {
            text.upsert_value(event.locale, event.name.clone());
            commands.entity(eid).insert(LastEditedBy(event.editor));
        },
        Err(QuerySingleError::NoEntities(..)) => {
            let text_id = ids.next();
            commands
            .spawn_text(text_id, (event.locale, event.name.clone()))
            .insert((
                ProjectName,
                LastEditedBy(event.editor)
            ));
        },

        Err(QuerySingleError::MultipleEntities(..)) => unreachable!()
    }

    commands.trigger(OpCompleted { op: event.op });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aspect::metadata::{BaseLocale, ProjectName};
    use crate::aspect::text::Text;
    use crate::engine_plugin::{EngineIdGen, IdEntityMap};

    fn setup() -> World {
        let mut world = World::new();
        world.init_resource::<EngineIdGen>();

        let locale_id = LocaleId::new(1, 0);
        let locale_eid = world.spawn((
            locale_id.as_untyped(),
            BaseLocale,
        )).id();

        let mut id_entity_map = IdEntityMap::default();
        id_entity_map.insert(locale_id, locale_eid);
        world.insert_resource(id_entity_map);

        world.add_observer(on_set_project_name);

        world
    }

    #[test]
    fn creates_name_when_missing() {
        let mut world = setup();
        let editor = EditorId::new(0xFFFF, 0);
        let op = OpId(0);

        world.trigger(SetProjectName {
            op,
            editor,
            locale: LocaleId::new(1, 0),
            name: "New Project".into(),
        });
        world.flush();

        let mut query = world.query_filtered::<(&Text, &LastEditedBy), With<ProjectName>>();
        let (text, edited_by) = query.single(&world).unwrap();
        assert_eq!(text.get_value(LocaleId::new(1, 0)), Some("New Project"));
        assert_eq!(edited_by.0, editor);
    }

    #[test]
    fn updates_existing_name() {
        let mut world = setup();
        let locale = LocaleId::new(1, 0);
        let text_id = world.resource::<EngineIdGen>().next_text();
        world.spawn((
            text_id.as_untyped(),
            Text::new(locale, "Old"),
            ProjectName,
        ));

        let editor = EditorId::new(0xFFFF, 0);
        world.trigger(SetProjectName {
            op: OpId(0),
            editor,
            locale,
            name: "Updated".into(),
        });
        world.flush();

        let mut q = world.query_filtered::<&Text, With<ProjectName>>();
        assert_eq!(q.single(&world).unwrap().get_value(locale), Some("Updated"));
    }
}