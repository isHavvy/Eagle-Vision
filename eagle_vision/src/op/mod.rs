#![warn(unused)]

use crate::bevy::prelude::*;

use eagle_squak::id::{EagleVisionId, Existing, New, SourceId};
use eagle_squak::protocol::{InvalidIdReason, RejectionReason};

use crate::engine_plugin::IdEntityMap;

pub mod set_project_name;

pub mod create_tag;
pub mod apply_tag;

pub mod create_comment;

pub struct OpPlugin;

impl Plugin for OpPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app
        .init_resource::<OpIdGen>()
        .add_observer(set_project_name::on_set_project_name)
        .add_observer(create_tag::on_create_attribute_definition)
        .add_observer(apply_tag::on_apply_tag)
        .add_observer(create_comment::on_create_comment)
        ;
    }
}

// eagle_vision/src/ops/mod.rs
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct OpId(pub u64);

#[derive(Debug, Default, Resource)]
pub struct OpIdGen(u64);

impl OpIdGen {
    pub fn next(&mut self) -> OpId {
        let id = OpId(self.0);
        self.0 += 1;
        id
    }
}

#[derive(Debug, Event)]
pub struct OpCompleted {
    pub op: OpId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpRejectionReason {
    IdNotFound(EagleVisionId),
    IdAlreadyExists(EagleVisionId),
    SourceNotOwned { attempted: SourceId, allowed: SourceId },
}

impl From<OpRejectionReason> for RejectionReason {
    fn from(reason: OpRejectionReason) -> Self {
        match reason {
            OpRejectionReason::IdAlreadyExists(id) => RejectionReason::IdAlreadyExists { id },
            OpRejectionReason::SourceNotOwned { attempted, allowed } => {
                RejectionReason::InvalidId(InvalidIdReason::SourceNotOwned { attempted_source: attempted, owned_source: allowed })
            },
            OpRejectionReason::IdNotFound(id) => RejectionReason::IdNotFound { id },
        }
    }
}

#[derive(Debug, Event)]
pub struct OpRejected {
    pub op: OpId,
    pub reason: OpRejectionReason,
}

pub trait UnvalidatedOp {
    type Trigger<'t>: Default;
    type ValidatedOp: for <'t> Event<Trigger<'t> = Self::Trigger<'t>>;

    fn check(self, ids: &IdEntityMap, source: SourceId) -> Result<Self::ValidatedOp, OpRejectionReason>;
}

pub trait UnvalidatedId {
    type ValidatedId;

    fn check(self, ids: &IdEntityMap, source: SourceId) -> Result<Self::ValidatedId, OpRejectionReason>;
}

impl<Id: Into<EagleVisionId> + Copy> UnvalidatedId for New<Id> {
    type ValidatedId = Id;

    /// Validates an id passed in by a client command to create an aspect.
    /// 
    /// Specifically, it checks that the id isn't already in use
    /// and that it is from the source that the session is allowed to mint ids from.
    fn check(self, ids: &IdEntityMap, source: SourceId) -> Result<Self::ValidatedId, OpRejectionReason> {
        let id = self.0.into();
        if ids.contains(id) {
            Err(OpRejectionReason::IdAlreadyExists(id))
        } else if id.source_id() != source {
            Err(OpRejectionReason::SourceNotOwned { attempted: id.source_id(), allowed: source })
        } else {
            Ok(self.0)
        }
    }
}

impl<Id: Into<EagleVisionId> + Copy> UnvalidatedId for Existing<Id> {
    type ValidatedId = Id;

    fn check(self, ids: &IdEntityMap, _source: SourceId) -> Result<Self::ValidatedId, OpRejectionReason> {
        if ids.contains(self.0.into()) {
            Ok(self.0)
        } else {
            Err(OpRejectionReason::IdNotFound(self.0.into()))
        }
    }
}

macro_rules! op_item {
    (
        $OpValidated:ident, $OpUnvalidated:ident {
            $( $field:ident : $kind:ident $ty:ty ),+ $(,)?
        }
    ) => {
        #[derive(Debug, bevy::prelude::Event)]
        pub struct $OpValidated {
            $( pub $field : op_item!(@validated_ty $kind $ty), )+
        }

        #[derive(Debug, Clone)]
        pub struct $OpUnvalidated {
            $( pub $field : op_item!(@unvalidated_ty $kind $ty), )+
        }

        impl crate::op::UnvalidatedOp for $OpUnvalidated {
            type Trigger<'t> = <$OpValidated as bevy::prelude::Event>::Trigger<'t>;
            type ValidatedOp = $OpValidated;

            fn check(
                self,
                ids: &crate::engine_plugin::IdEntityMap,
                source: eagle_squak::id::SourceId
            )
            -> Result<Self::ValidatedOp, crate::op::OpRejectionReason>
            {
                Ok($OpValidated {
                    $( $field: op_item!(@check self.$field, $kind, $ty, ids, source), )+
                })
            }
        }
    };

    (@validated_ty new $ty:ty) => { $ty };
    (@validated_ty existing $ty:ty) => { $ty };
    (@validated_ty valid $ty:ty) => { $ty };

    (@unvalidated_ty new $ty:ty) => { eagle_squak::id::New<$ty> };
    (@unvalidated_ty existing $ty:ty) => { eagle_squak::id::Existing<$ty> };
    (@unvalidated_ty valid $ty:ty) => { $ty };

    (@check $field:expr, new, $ty:ty, $ids:expr, $source:expr) => { crate::op::UnvalidatedId::check($field, $ids, $source)? };
    (@check $field:expr, existing, $ty:ty, $ids:expr, $source:expr) => { crate::op::UnvalidatedId::check($field, $ids, $source)? };
    (@check $field:expr, valid, $ty:ty, $ids:expr, $source:expr) => { $field };
}

use op_item;

#[cfg(test)]
pub(crate) mod test_util {
    use bevy::prelude::*;
    use eagle_squak::id::{EagleVisionId, EditorId, LocaleId, SourceId, TextId};
    use eagle_squak::id_allocator::AspectIdAllocator;

    use crate::aspect::text::Text;
    use crate::engine_plugin::IdEntityMap;

    /// The source a test "client" mints from, as a handshake would allocate.
    pub(crate) const CLIENT: SourceId = SourceId(2);
    /// A different source, for ownership-rejection tests.
    #[expect(unused)] pub(crate) const STRANGER: SourceId = SourceId(3);
    /// Core library source, where prelude data lives.
    pub(crate) const CORE: SourceId = SourceId(1);

    pub(crate) const EDITOR: EditorId = EditorId::new(0xFFFF, 0);
    pub(crate) const EN: LocaleId = LocaleId::new(1, 0);

    pub(crate) fn client() -> AspectIdAllocator { AspectIdAllocator::new(CLIENT) }
    #[expect(unused)] pub(crate) fn stranger() -> AspectIdAllocator { AspectIdAllocator::new(STRANGER) }

    /// Spawn an entity and register it, standing in for what an op's
    /// spawn helper + MapId would do.
    pub(crate) fn spawn_mapped(
        world: &mut World,
        id: impl Into<EagleVisionId>,
        bundle: impl Bundle,
    ) -> Entity {
        let id = id.into();
        let entity = world.spawn((id, bundle)).id();
        world.resource_mut::<IdEntityMap>().insert(id, entity);
        entity
    }

    /// A Text aspect that exists, for `existing TextId` fields.
    pub(crate) fn spawn_text(world: &mut World, id: TextId, value: &str) -> Entity {
        spawn_mapped(world, id, Text::new(EN, value))
    }

    pub(crate) fn world_with_map() -> World {
        let mut world = World::new();
        world.init_resource::<IdEntityMap>();
        world
    }

    /// An id that was minted but never spawned. For `IdNotFound` tests.
    pub(crate) fn ghost(source: SourceId, aspect: eagle_squak::aspect::AspectId) -> EagleVisionId {
        EagleVisionId::new(source.get(), aspect, 999)
    }
}