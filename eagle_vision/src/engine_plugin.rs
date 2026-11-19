use std::num::NonZeroU16;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::ecs::system::SystemParam;
use bevy::platform::collections::HashMap;
use eagle_squak::id::{EagleVisionId, LocaleId, SourceId};
use eagle_squak::id_allocator::AspectIdAllocator;
use smallvec::SmallVec;

use crate::bevy::prelude::*;

use crate::private::EagleVisionSealed;
use crate::schedule::{PostStartup, Startup, Update};
use crate::subscription::SubscriptionPlugin;
use crate::op::OpPlugin;

pub struct EagleVisionEnginePlugin;

impl Plugin for EagleVisionEnginePlugin {
    fn build(&self, app: &mut App) {
        app
            .init_schedule(Startup)
            .init_schedule(PostStartup)
            .init_schedule(Update)

            .init_resource::<EngineIdGen>()
            .init_resource::<IdEntityMap>()
            .init_resource::<SourceAllocator>()
            .init_resource::<WorkPending>()

            .add_plugins((
                OpPlugin,
                SubscriptionPlugin
            ))
        ;
    }
}

/// The [`EagleVisionId`](eagle_squak::id::EagleVisionId) generator for the server.
#[derive(Resource)]
pub struct EngineIdGen(pub AspectIdAllocator);

impl EngineIdGen {
    // As per Squak, 0 is reserved for the project's source files.
    // Could make this 1, but then all dependencies will probably be at 3 or higher.
    // Making this 0xFFFF makes it 2 or higher, but if clients are allocated going
    // down, then dependencies could have sources 1 through however many is needed.
    //
    // No matter what, we'll need a resource that keeps track of which source id
    // is which source once we start implementing dependencies or multiple clients.
    const ENGINE_SOURCE: SourceId = SourceId(-1i16 as u16);

    fn new() -> Self {
        Self(AspectIdAllocator::new(Self::ENGINE_SOURCE))
    }
}

impl Default for EngineIdGen {
    fn default() -> Self {
        Self::new()
    }
}

impl std::ops::Deref for EngineIdGen {
    type Target = AspectIdAllocator;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Default, Resource)]
pub struct IdEntityMap(HashMap<EagleVisionId, Entity>);

impl IdEntityMap {
    pub fn insert(&mut self, id: impl Into<EagleVisionId>, entity: Entity) {
        self.0.insert(id.into(), entity);
    }

    pub fn get(&self, id: impl Into<EagleVisionId>) -> Option<Entity> {
        self.0.get(&id.into()).cloned()
    }

    pub fn contains(&self, id: impl Into<EagleVisionId>) -> bool {
        self.0.contains_key(&id.into())
    }
}

#[derive(Debug, Resource)]
pub struct SourceAllocator(NonZeroU16);

impl SourceAllocator {
    pub fn new() -> Self {
        // 0 is reserved for the project source file.
        // 1 is reserved for the core library.
        // Thus we start at 2.
        Self(2.try_into().unwrap())
    }

    pub fn next(&mut self) -> Option<NonZeroU16> {
        // 0xFFFF is reserved for the EagleVision server.
        if self.0.get() == 0xFFFF {
            None
        } else {
            let ret = Some(self.0);
            self.0 = self.0.saturating_add(1);
            ret
        }
    }
}

impl Default for SourceAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Resource, Default)]
pub(crate) struct WorkPending(pub AtomicBool);

impl WorkPending {
    pub fn set(&self) {
        self.0.store(true, Ordering::Relaxed);
    }

    pub fn clear(&self) {
        self.0.store(false, Ordering::Relaxed);
    }

    pub fn is_set(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

/// Single entity marker that must exist
/// 
/// A component that implements `GuaranteedMarkerComponent` must exist on
/// a single entity after the [`Setup`](crate::schedule::Setup) schedule of an EagleVision world.
/// 
/// This trait cannot be implemented outside of the crate that defines it.
pub trait GuaranteedMarkerComponent : Component + Sized + EagleVisionSealed {
    fn get(world: &World) -> Entity {
        world.try_query_filtered::<Entity, With<Self>>().unwrap().single(world).unwrap()
    }
}

#[derive(Debug, Clone, Component)]
pub(crate) struct View {
    /// Locale preference order for localized text projection.
    /// 
    /// Guaranteed to be non-empty since the project's base locale is always included.
    pub locales: SmallVec<[LocaleId; 2]>,
}

impl View {
    pub fn first_locale(&self) -> LocaleId {
        self.locales[0]
    }
}

/// An Entity that has a View component.
/// 
/// It's not possible to cheaply track this, so it is up to
/// construction sites to enforce this invariant. View components
/// are never removed once added, so storing them isn't a problem.
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct ViewEntity(Entity);

impl ViewEntity {
    pub fn new_unchecked(entity: Entity) -> Self {
        Self(entity)
    }

    pub fn inner(self) -> Entity {
        self.0
    }

    pub(crate) fn view_from_world(self, world: &World) -> &View {
        world.get(self.0).unwrap()
    }
}