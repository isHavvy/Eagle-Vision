//! Concrete data of the language e.g. a name, or function
//! 
//! Things are the part of the language that you'd find in the
//! source code of other languages. For example, items in Rust,
//! expressions, names.
//! 
//! Name "thing" is subject to change.

use bevy::prelude::*;
use eagle_squak::aspect::AspectId::Ty;
use eagle_squak::id::*;

use crate::aspect::editor::Editor;
use crate::aspect::locale::Locale;
use crate::aspect::text::Text;

pub mod editor;
pub mod expression;
pub mod function;
pub mod locale;
pub mod metadata;
pub mod text;
pub mod ty;
pub mod exe;

pub trait AspectCommandsExt<'w, 's> {
    fn spawn_locale(&mut self, id: LocaleId, code: impl Into<String>) -> EntityCommands<'_>;
    fn spawn_text(&mut self, id: TextId, value: (LocaleId, impl Into<String>)) -> EntityCommands<'_>;
    fn spawn_ty(&mut self, id: TyId,) -> EntityCommands<'_>;
    fn spawn_editor(&mut self, id: EditorId, username: TextId) -> EntityCommands<'_>;
}

impl<'w, 's> AspectCommandsExt<'w, 's> for Commands<'w, 's> {
    fn spawn_locale(&mut self, id: LocaleId, code: impl Into<String>) -> EntityCommands<'_> {
        let code = code.into();
        self.spawn((
            id.as_untyped(),
            Locale(code),
        ))
    }

    fn spawn_text(&mut self, id: TextId, (locale_id, value): (LocaleId, impl Into<String>)) -> EntityCommands<'_> {
        self.spawn((
            id.as_untyped(),
            Text::new(locale_id, value.into())
        ))
    }

    fn spawn_ty(&mut self, id: TyId) -> EntityCommands<'_> {
        self.spawn((
            id.as_untyped(),
            Ty
        ))
    }

    fn spawn_editor(&mut self, id: EditorId, username: TextId) -> EntityCommands<'_> {
        self.spawn((
            id.as_untyped(),
            Editor { username }
        ))
    }
}