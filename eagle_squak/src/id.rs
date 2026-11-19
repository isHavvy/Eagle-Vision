// TODO(Havvy): Serialization is left unimplemented on purpose since it's not useful yet.

use std::{marker::PhantomData, num::NonZeroU64};

use crate::aspect::{AspectError, AspectId, AspectKind, Function};

/// An ID of an aspect of an Eagle Vision source.
///
/// The ID stores the source of the id and which aspect it is.
/// It does so by storing the source in the first 16 bits and
/// the aspect in the following 8 bits. The final 40 bits are
/// the address space for that aspect/source combination.
/// 
/// This ID can never be zero. This is ensured because the aspect
/// value cannot be zero.
///
/// There are various kinds of sources:
///
/// 1. File - Aspects defined by this project loaded from the source file.
/// 2. Language Server - Aspects created by the language server.
/// 3. Client - Aspects created by a client (e.g. an editor) connected to the server.
/// 4. Dependencies - Each dependency gets its own source id.
///
/// Note that #2 and #3 are ephermeral. These source IDs get turned into file sources when saved.
/// Which source id comes from which kind of source is unspecified.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
pub struct EagleVisionId(NonZeroU64);

impl EagleVisionId {
    pub const ADDRESS_BITS: u32 = 40;
    pub const ASPECT_BITS: u32 = 8;
    pub const SOURCE_BITS: u32 = 16;

    const ASPECT_SHIFT: u32 = Self::ADDRESS_BITS;
    const SOURCE_SHIFT: u32 = Self::ASPECT_SHIFT + Self::ASPECT_BITS;

    const ADDRESS_MASK: u64 = (1 << Self::ADDRESS_BITS) - 1; // 0x0000_00FF_FFFF_FFFF

    #[inline]
    pub const fn new(source: u16, aspect: AspectId, address: u64) -> Self {
        debug_assert!(address <= Self::ADDRESS_MASK, "Address space overflowed.");

        let id = ((source as u64) << Self::SOURCE_SHIFT)
            | ((aspect as u64) << Self::ASPECT_SHIFT)
            | (address & Self::ADDRESS_MASK);

        // SAFETY: `aspect` is non-zero, so aspect's bits are always non-zero.
        let id = unsafe { NonZeroU64::new_unchecked(id) };
        Self(id)
    }

    #[inline]
    pub fn from_raw(raw: u64) -> Result<Self, AspectError> {
        let raw_aspect = (raw >> Self::ASPECT_SHIFT) as u8; // as cast automatically ignores higher bits
        AspectId::from_u8(raw_aspect).map(|_| NonZeroU64::new(raw).unwrap()).map(EagleVisionId) // valid aspect ensures raw is non-zero
    }

    #[inline]
    pub const fn as_raw(self) -> u64 {
        self.0.get()
    }

    #[inline]
    pub const fn source(self) -> u16 {
        ((self.0.get()) >> Self::SOURCE_SHIFT) as u16
    }

    #[inline]
    pub const fn aspect(self) -> AspectId {
        let raw_aspect = (self.0.get() >> Self::ASPECT_SHIFT) as u8; // as cast automatically ignores higher bits
        
        // SAFETY: Construction of an ID already checks the aspect is valid.
        unsafe { AspectId::new_unchecked(raw_aspect) }
    }

    #[inline]
    pub const fn address(self) -> u64 {
        self.0.get() & Self::ADDRESS_MASK
    }
}

impl std::fmt::Debug for EagleVisionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "EV_ID(source={}, {:?}, addr={})",
            self.source(),
            self.aspect(),
            self.address()
        )
    }
}

/// An EagleVision ID that has a checked aspect.
#[repr(transparent)]
pub struct AspectedId<A: AspectKind> {
    id: EagleVisionId,
    _kind: PhantomData<fn() -> A>,
}

// Derives would impl A: AspectKind + Clone. The clone bound is not needed.
impl<A: AspectKind> Clone for AspectedId<A> {
    fn clone(&self) -> Self {
        Self { id: self.id.clone(), _kind: PhantomData }
    }
}

impl<A: AspectKind> Copy for AspectedId<A> {}

impl<A: AspectKind> std::fmt::Debug for AspectedId<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}Id(source={}, addr={})", A::ID, self.source(), self.address())
    }
}

impl<A: AspectKind> PartialEq for AspectedId<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<A: AspectKind> Eq for AspectedId<A> {}

impl<A: AspectKind> PartialOrd for AspectedId<A> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.id.partial_cmp(&other.id)
    }
}

impl<A: AspectKind> Ord for AspectedId<A> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl<A: AspectKind> std::hash::Hash for AspectedId<A> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<A: AspectKind> AspectedId<A> {
    #[inline]
    pub const fn new(source: u16, address: u64) -> Self {
        Self {
            id: EagleVisionId::new(source, A::ID, address),
            _kind: PhantomData,
        }
    }

    /// Reinterpret a raw id as this aspect, checking the tag.
    #[inline]
    pub fn from_untyped(id: EagleVisionId) -> Result<Self, AspectError> {
        if id.aspect() == A::ID {
            Ok(Self { id, _kind: PhantomData })
        } else {
            Err(AspectError::AspectMismatch { expected: A::ID, found: id.aspect() })
        }
    }

    /// # Safety
    /// `id.aspect()` must equal `A::ID`.
    #[inline]
    pub unsafe fn from_untyped_unchecked(id: EagleVisionId) -> Self {
        debug_assert_eq!(id.aspect(), A::ID);
        Self { id, _kind: PhantomData }
    }

    #[inline]
    pub fn from_raw(raw: u64) -> Result<Self, AspectError> {
        EagleVisionId::from_raw(raw).and_then(Self::from_untyped)
    }

    #[inline]
    pub const fn as_untyped(self) -> EagleVisionId { self.id }

    #[inline]
    pub const fn as_raw(self) -> u64 { self.id.as_raw() }

    #[inline]
    pub const fn source(self) -> u16 { self.id.source() }

    #[inline]
    pub const fn address(self) -> u64 { self.id.address() }
}

impl<A: AspectKind> From<AspectedId<A>> for EagleVisionId {
    fn from(value: AspectedId<A>) -> Self {
        value.as_untyped()
    }
}

impl<A: AspectKind> TryFrom<EagleVisionId> for AspectedId<A> {
    type Error = AspectError;
    
    fn try_from(id: EagleVisionId) -> Result<Self, Self::Error> {
        if id.aspect() == A::ID {
            Ok(Self {
                id,
                _kind: PhantomData,
            })
        } else {
            Err(AspectError::AspectMismatch { expected: A::ID, found: id.aspect() })
        }
    }
}

impl<A: AspectKind> TryFrom<u64> for AspectedId<A> {
    type Error = AspectError;
    
    fn try_from(raw: u64) -> Result<Self, Self::Error> {
        EagleVisionId::from_raw(raw).and_then(|id| id.try_into())
    }
}

impl<A: AspectKind> PartialEq<EagleVisionId> for AspectedId<A> {
    fn eq(&self, other: &EagleVisionId) -> bool {
        self.as_untyped() == *other
    }
}

impl<A: AspectKind> PartialEq<AspectedId<A>> for EagleVisionId {
    fn eq(&self, other: &AspectedId<A>) -> bool {
        *self == other.as_untyped()
    }
}

pub type ExecutableId = AspectedId<crate::aspect::Executable>;
pub type FunctionId = AspectedId<crate::aspect::Function>;
pub type ExpressionId = AspectedId<crate::aspect::Expression>;
pub type TyId = AspectedId<crate::aspect::Ty>;
pub type TextId = AspectedId<crate::aspect::Text>;
pub type LocaleId = AspectedId<crate::aspect::Locale>;
pub type ParameterId = AspectedId<crate::aspect::Parameter>;
pub type EditorId = AspectedId<crate::aspect::Editor>;

const _: () = assert!(EagleVisionId::ADDRESS_BITS + EagleVisionId::ASPECT_BITS + EagleVisionId::SOURCE_BITS == 64);
const _: () = assert!(size_of::<EagleVisionId>() == 8);
const _: () = assert!(size_of::<Option<EagleVisionId>>() == 8);
const _: () = assert!(size_of::<AspectedId<Function>>() == 8);
const _: () = assert!(size_of::<Option<AspectedId<Function>>>() == 8);

const _: () = {
    fn assert_trait_impls<T: Copy + Clone + std::fmt::Debug + Eq + PartialEq + std::hash::Hash + Ord + PartialOrd>() {}
    let _ = assert_trait_impls::<AspectedId<Function>>;
};