#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AspectError {
    /// Zero is explicitly not allowed.
    Zero,

    /// Aspect number not defined in this version.
    UnknownAspect(u8),

    AspectMismatch { expected: AspectId, found: AspectId },
}

impl std::fmt::Display for AspectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AspectError::Zero => write!(f, "aspect cannot be zero"),
            AspectError::UnknownAspect(value) => write!(f, "unknown aspect {}", value),
            AspectError::AspectMismatch { expected, found } => write!(f, "expected aspect {}, found {}", expected, found),
        }
    }
}

impl std::error::Error for AspectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

// A data declaration for each aspect named must also exist in this file.
macro_rules! aspects {
    ($($name:ident = $disc:literal),+ $(,)?) => {
        /// ID for an aspect of an Eagle Vision source
        /// 
        /// Discriminants are guaranteeds to be non-zero. Unsafe code may depend on this.
        /// 
        /// It is a breaking change for a discriminant value to change. New values may be added.
        #[repr(u8)]
        #[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
        #[cfg_attr(feature = "bevy", derive(bevy_ecs::component::Component))]
        pub enum AspectId { $($name = $disc),+ }

        impl AspectId {
            pub const VARIANTS: u8 = Self::ALL.len() as u8;
            pub const ALL: &'static [Self] = &[$(Self::$name),+];
            pub const fn from_u8(v: u8) -> Result<Self, AspectError> {
                match v {
                    0 => Err(AspectError::Zero),
                    $($disc => Ok(Self::$name),)+
                    v => Err(AspectError::UnknownAspect(v)) }
            }
        }

        impl std::fmt::Display for AspectId {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                let name = match self {
                    $(Self::$name => stringify!($name),)+
                };
                f.pad(name)
            }
        }

        $(
            impl AspectKind for $name { const ID: AspectId = AspectId::$name; }
        )+
    };
}

// When adding a new aspect, remember to also:
// 1. Add a new ID type alias in id.rs
// 2. Add a reexport of the ID in lib.rs
// 3. Add a method in id_allocator.rs
aspects! {
    Executable = 1,
    Function = 2,
    Expression = 3,
    Ty = 4,
    Text = 5,
    Locale = 6,
    Parameter = 7,
    Editor = 8,
}

/// ID associated to every EagleVision aspect.
impl AspectId {
    /// # Safety
    /// 
    /// `aspect` must be a valid discriminant of AspectId.
    #[inline]
    pub const unsafe fn new_unchecked(aspect: u8) -> Self {
        debug_assert!(Self::from_u8(aspect).is_ok());
        // SAFETY: repr(u8) guarantees size; caller guarantees a valid discriminant
        unsafe { *(&aspect as *const u8 as *const Self) }
    }
}

impl TryFrom<u8> for AspectId {    
    type Error = AspectError;
    
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        AspectId::from_u8(value)
    }
}

pub trait AspectKind {
    const ID: AspectId;
}

#[derive(Debug)]
pub struct Executable;

#[derive(Debug)]
pub struct Function;

#[derive(Debug)]
pub struct Expression;

#[derive(Debug)]
pub struct Ty;

#[derive(Debug)]
pub struct Text;

#[derive(Debug)]
pub struct Locale;

#[derive(Debug)]
pub struct Parameter;

#[derive(Debug)]
pub struct Editor;

const _ALL_ASPECT_ID_DISCRIMINANTS_ARE_NON_ZERO: () = {
    let mut ix = 0;
    while ix < AspectId::ALL.len() {
        assert!(AspectId::ALL[ix] as u8 != 0);
        ix += 1;
    }
};