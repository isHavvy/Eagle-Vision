//! Internal Bevy Interface

/// Internal prelude
/// 
/// Some exports e.g. `Text` and `Query` clash with types named in this project.
/// Instead of trying to find different names, we either skip it in the prelude
/// or rename it such as BevyQuery.
// Re-exports are alphabetical by Bevy name if you assume spaces between words.
// So ResMut -> "Res Mut" so it's before "Resource".
pub mod prelude {
    pub use bevy::prelude::{
        App,
        Changed,
        Component,
        Entity,
        EntityWorldMut,
        Event,
        IntoScheduleConfigs,
        Message,
        MessageReader,
        MessageWriter,
        Mut,
        NonSendMut,
        On,
        Plugin,
        Query as BevyQuery,
        Res,
        ResMut,
        Resource,
        System,
        With,
        World,
    };

    pub use super::Commands;
}

use std::ops::{Deref, DerefMut};

pub use bevy::app::App;
pub use bevy::ecs::schedule::SystemSet;

pub mod commands {
    pub use bevy::prelude::{
        Command,
        EntityCommand,
        EntityCommands,
    };
}

/// EagleVision's wrapper of Bevy's Commands.
#[derive(bevy::ecs::system::SystemParam)]
pub struct Commands<'w, 's> {
    inner: bevy::prelude::Commands<'w, 's>,
}

impl<'w, 's> Commands<'w, 's> {
    pub fn new(inner: bevy::prelude::Commands<'w, 's>) -> Self {
        Self { inner }
    }
}

impl<'w, 's> Commands<'w, 's> {
    pub fn reborrow(&mut self) -> Commands<'w, '_> {
        Commands::new(self.inner.reborrow())
    }
}

impl<'w, 's> Deref for Commands<'w, 's> {
    type Target = bevy::prelude::Commands<'w, 's>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'w, 's> DerefMut for Commands<'w, 's> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}