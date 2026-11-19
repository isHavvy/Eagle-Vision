use bevy::prelude::*;
use bevy::ecs::schedule::ScheduleLabel;
use eagle_squak::id::{EditorId, LocaleId};
use eagle_squak::id_allocator::SourceIdAllocator;

use crate::aspect::AspectCommandsExt;
use crate::aspect::editor::{EditorAnonymous, EditorErased};
use crate::aspect::text::AspectName;
use crate::aspect::metadata::{BaseLocale, ProjectName};
use crate::eaglevision::ServerIdGen;

#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProjectInit;

#[derive(Debug, Clone, Default, Resource)]
pub struct EagleVisionInitialSettings {
    pub base_locale: Option<String>,
    pub project_name: Option<String>,
    pub creator: Option<String>,
}

impl EagleVisionInitialSettings {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn set_project_name(&mut self, name: impl Into<String>) -> &mut Self {
        let name = name.into();

        self.project_name = if name.len() > 0 {
            Some(name)
        } else {
            None
        };

        self
    }

    pub fn set_base_locale(&mut self, locale: impl Into<String>) -> &mut Self {
        let locale = locale.into();

        self.base_locale = if locale.len() > 0 {
            Some(locale)
        } else {
            None
        };

        self
    }

    pub fn set_creator(&mut self, creator: impl Into<String>) -> &mut Self {
        let creator = creator.into();

        self.creator = if creator.len() > 0 {
            Some(creator)
        } else {
            None
        };

        self
    }
}

/// Plugin for initializing a new EagleVision project.
pub struct EagleVisionInitializationPlugin {
    pub settings: EagleVisionInitialSettings,
}

impl Plugin for EagleVisionInitializationPlugin {
    fn build(&self, app: &mut App) {
        app
        .insert_resource(self.settings.clone())
        .add_systems(Startup, init_project)
        ;
    }
}

fn init_project(
    settings: Res<EagleVisionInitialSettings>,
    ids: Res<ServerIdGen>,
    mut commands: Commands,
) {
    bevy::log::debug!("Project is being initalized");

    // We are initializing a core lib alongside the project.
    // As such, we need its own source ids for it.
    let core_ids = SourceIdAllocator::new(1);

    let erased_editor_id = core_ids.next_editor();

    let en_locale = core_ids.next_locale();
    let en_eid = commands.spawn_locale(en_locale, "en-US").id();

    commands.insert_resource(CoreLocale(en_eid));

    initalize_core_tys(commands.reborrow(), &core_ids, en_locale);

    let base_locale = match settings.base_locale.clone() {
        Some(code) => {
            let base_locale_id = ids.next_locale();
            commands.spawn_locale(base_locale_id, code).insert(BaseLocale);
            base_locale_id
        },
        None => {
            commands.entity(en_eid).insert(BaseLocale);  
            en_locale
        },
    };

    if let Some(project_name) = settings.project_name.clone() {
        let name_id = ids.next_text();
         commands.spawn_text(name_id, (base_locale, project_name)).insert(ProjectName);
    }

    initialize_core_editors(commands.reborrow(), &core_ids, en_locale, erased_editor_id);
}

/// The locale core tys and diagnostics are authored in. Always en-US.
#[derive(Resource)]
pub struct CoreLocale(pub Entity);

#[inline]
fn initialize_core_ty<Ty: Default + Component>
(
    commands: &mut Commands,
    ids: &SourceIdAllocator,
    en: LocaleId,
    name: &'static str,
) {
    let name_id = ids.next();
    commands.spawn_text(name_id, (en, name));
    commands.spawn_ty(ids.next()).insert((
        AspectName(name_id),
        Ty::default()
    ));
}

fn initalize_core_tys(
    mut commands: Commands,
    ids: &SourceIdAllocator,
    en: LocaleId
) {
    use crate::aspect::ty::*;
    let commands = &mut commands;

    initialize_core_ty::<TyUnit>(commands, ids, en, "Unit");
    initialize_core_ty::<TyUnknown>(commands, ids, en, "Unknown Type");
    initialize_core_ty::<TyInt32>(commands, ids, en, "Int32");
    initialize_core_ty::<TyFloat64>(commands, ids, en, "Float64");
    initialize_core_ty::<TyBoolean>(commands, ids, en, "Boolean");
    initialize_core_ty::<TyUtf8>(commands, ids, en, "Utf8");
    initialize_core_ty::<TyFunction>(commands, ids, en, "Function");
}

fn initialize_core_editors(
    mut commands: Commands,
    ids: &SourceIdAllocator,
    en: LocaleId,
    erased_id: EditorId,
) {
    let text_id = ids.next_text();
    commands.spawn_text(text_id, (en, "(anonymous editor)"));
    commands.spawn_editor(ids.next(), text_id).insert(EditorAnonymous);

    let text_id = ids.next_text();
    commands.spawn_text(text_id, (en, "(erased editor)"));
    commands.spawn_editor(erased_id, text_id).insert(EditorErased);
}