//! Concrete data of the language e.g. a name, or function
//! 
//! Things are the part of the language that you'd find in the
//! source code of other languages. For example, items in Rust,
//! expressions, names.
//! 
//! Name "thing" is subject to change.

use crate::bevy::prelude::*;
use crate::bevy::commands::{EntityCommands};
use bevy::prelude::EntityCommand;
use eagle_squak::id::*;

use crate::aspect::comment::{Comment, CommentOn};
use crate::aspect::editor::Editor;
use crate::aspect::locale::Locale;
use crate::aspect::attribute::{Attribute, AttributeKind};
use crate::aspect::text::Text;
use crate::aspect::ty::Ty;
use crate::engine_plugin::IdEntityMap;

pub mod editor;
pub mod expression;
pub mod function;
pub mod locale;
pub mod metadata;
pub mod text;
pub mod ty;
pub mod exe;
pub mod attribute;
pub mod comment;

impl<'w, 's> Commands<'w, 's> {
    pub fn spawn_locale(&mut self, id: LocaleId, code: impl Into<String>) -> EntityCommands<'_> {
        let code = code.into();

        let mut commands = self.spawn((
            id.as_untyped(),
            Locale(code),
        ));
        commands.queue(MapId(id.as_untyped()));
        commands
    }

    pub fn spawn_text(&mut self, id: TextId, (locale_id, value): (LocaleId, impl Into<String>)) -> EntityCommands<'_> {
        let value = value.into();
        let mut commands = self.spawn((
            id.as_untyped(),
            Text::new(locale_id, value)
        ));
        commands.queue(MapId(id.as_untyped()));
        commands
    }

    pub fn spawn_ty(&mut self, id: TyId) -> EntityCommands<'_> {
        let mut commands = self.spawn((
            id.as_untyped(),
            Ty
        ));
        commands.queue(MapId(id.as_untyped()));
        commands
    }

    pub fn spawn_editor(&mut self, id: EditorId, username: TextId) -> EntityCommands<'_> {
        let mut commands = self.spawn((
            id.as_untyped(),
            Editor { username }
        ));
        commands.queue(MapId(id.as_untyped()));
        commands
    }

    pub fn spawn_attribute_definition(&mut self, id: AttributeDefinitionId) -> EntityCommands<'_> {
        let mut commands = self.spawn((
            id.as_untyped(),
            AttributeKind {}
        ));
        commands.queue(MapId(id.as_untyped()));
        commands
    }

    pub fn spawn_attribute(&mut self, id: AttributeId, kind: AttributeDefinitionId, on: EagleVisionId) -> EntityCommands<'_> {
        let mut commands = self.spawn((
            id.as_untyped(),
            Attribute { kind, on },
        ));
        commands.queue(MapId(id.as_untyped()));
        commands
    }

    pub fn spawn_comment(&mut self, id: CommentId, body: TextId, on: EagleVisionId) -> EntityCommands<'_> {
        let mut commands = self.spawn((
            id.as_untyped(),
            Comment { body },
            CommentOn(on),
        ));
        commands.queue(MapId(id.as_untyped()));
        commands
    }
}

struct MapId(EagleVisionId);

impl EntityCommand for MapId {
    type Out = ();
    
    fn apply(self, entity: EntityWorldMut) -> Self::Out {
        let eid = entity.id();
        let world = entity.into_world_mut();
        world.resource_mut::<IdEntityMap>().insert(self.0, eid);
    }
}