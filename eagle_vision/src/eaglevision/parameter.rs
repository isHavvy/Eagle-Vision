use bevy::prelude::*;

use crate::eaglevision::EagleVision;
use crate::aspect::ty::ActualTy;

/// A reference to a specific function parameter and the entire EagleVision source.
pub struct EagleVisionParameter<'ev>(EntityRef<'ev>);

impl<'ev> EagleVisionParameter<'ev> {
    pub(super) fn new(ev: &'ev EagleVision, parameter: Entity) -> Self {
        Self(ev.app.world().entity(parameter))
    }

    pub fn ty(&self) -> Entity {
        self.0.get::<ActualTy>().unwrap().entity()
    }
}