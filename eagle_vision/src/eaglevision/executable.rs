use std::ops::{Deref, DerefMut};

use bevy::prelude::*;

use crate::bytecode;
use crate::bytecode::compile::CompileError;
use crate::eaglevision::{EagleVision, EntityType, HasComponent};
use crate::aspect::exe::Executable;

/// An [`Entity`] that has an [`Executable`] component.
pub struct ExecutableEntity(Entity);

impl EntityType for ExecutableEntity {}
impl HasComponent<Executable> for ExecutableEntity {}

// pub struct EagleVisionExecutable<'ev, S: AsEntityRef<'ev>> {
//     inner: EagleVisionEntity<'ev, S, ExecutableEntity>
// }

// impl<'ev> EagleVisionExecutable<'ev, SharedStorage<'ev>> {
//     pub fn new(ev: &'ev EagleVision, entity: Entity) -> Self {
//         Self { inner: EagleVisionEntity::new(ev, entity) }
//     }
// }

// impl<'ev> EagleVisionExecutable<'ev, MutableStorage<'ev>> {
//     pub fn new(ev: &'ev mut EagleVision, entity: Entity) -> Self {
//         Self { inner: EagleVisionEntity::new_mut(ev, entity) }
//     }
// }

// impl<'ev, S: AsEntityRef<'ev>> Deref for EagleVisionExecutable<'ev, S> {
//     type Target = EagleVisionEntity<'ev, S, ExecutableEntity>;

//     fn deref(&self) -> &Self::Target {
//         &self.inner
//     }
// }

// impl<'ev> DerefMut for EagleVisionExecutable<'ev, MutableStorage<'ev>> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.inner
//     }
// }

// impl<'ev, S: AsEntityRef<'ev>> EagleVisionExecutable<'ev, S> {
//     pub fn main_block(&self) -> Entity {
//         self.get_known::<Executable>().main_block
//     }
// }

// impl<'ev> EagleVisionExecutable<'ev, MutableStorage<'ev>> {
//     pub fn compile(&mut self) -> Result<Entity, CompileError> {
//         bytecode::compile::compile_executable(self)
//     }
// }