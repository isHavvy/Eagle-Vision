//! Executable

use bevy::prelude::*;

#[derive(Debug, Component)]
#[require(Name)]
pub struct Executable {
    /// Block expression that will be executed
    pub(crate) main_block: Entity,
}