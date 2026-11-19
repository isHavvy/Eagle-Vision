use std::sync::atomic::{AtomicU64, Ordering};
use crate::protocol::ServerWelcome;
use crate::id::{AspectedId, EagleVisionId, SourceId};
use crate::aspect_ids::*;
use crate::aspect::{AspectId, AspectKind};

/// Utility allocator of IDs for a given source.
/// 
/// A client will want to create one of these after a [`ServerWelcome`] message gives
/// it their `allocated_source`.
pub struct AspectIdAllocator {
    source: u16,
    counters: [AtomicU64; AspectId::VARIANTS as usize], // One counter per NodeCategory
}

impl AspectIdAllocator {
    pub const fn new(source: SourceId) -> Self {
        Self {
            source: source.get(),
            counters: [const { AtomicU64::new(0) }; _],
        }
    }

    pub fn new_from_server_welcome(welcome: ServerWelcome) -> Option<Self> {
        welcome.allocated_source.map(|id| Self::new(SourceId::from_nonzero(id)))
    }

    #[inline]
    pub fn next_with_aspect(&self, aspect: AspectId) -> EagleVisionId {
        let counter_ix = (aspect as usize) - 1;
        let addr = self.counters[counter_ix].fetch_add(1, Ordering::Relaxed);
        assert!(addr < (1 << 40), "40-bit address space exhausted for {:?}", aspect);
        EagleVisionId::new(self.source, aspect, addr)
    }

    #[inline]
    pub fn next<A: AspectKind>(&self) -> AspectedId<A> {
        let counter_ix = A::ID as usize - 1;
        let addr = self.counters[counter_ix].fetch_add(1, Ordering::Relaxed);
        assert!(addr < (1 << 40), "40-bit address space exhausted for {:?}", A::ID);
        AspectedId::new(self.source, addr)
    }

    #[inline]
    pub fn next_executable(&self) -> ExecutableId {
        self.next()
    }

    #[inline]
    pub fn next_function(&self) -> FunctionId {
        self.next()
    }

    #[inline]
    pub fn next_expression(&self) -> ExpressionId {
        self.next()
    }

    #[inline]
    pub fn next_ty(&self) -> TyId {
        self.next()
    }

    #[inline]
    pub fn next_text(&self) -> TextId {
        self.next()
    }

    #[inline]
    pub fn next_locale(&self) -> LocaleId {
        self.next()
    }

    #[inline]
    pub fn next_parameter(&self) -> ParameterId {
        self.next()
    }

    #[inline]
    pub fn next_editor(&self) -> EditorId {
        self.next()
    }

    #[inline]
    pub fn next_attribute_definition(&self) -> AttributeDefinitionId {
        self.next()
    }

    #[inline]
    pub fn next_attribute(&self) -> AttributeId {
        self.next()
    }

    #[inline]
    pub fn next_comment(&self) -> CommentId {
        self.next()
    }
}
