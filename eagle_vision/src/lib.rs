#![allow(unused)]

pub mod bytecode;
pub mod eaglevision;
pub mod diagnostic;
pub mod aspect;
pub mod server;
pub mod init;
pub mod subscription;

mod private {
    /// Trait only implementable in the eagle_vision crate.
    /// 
    /// A trait that has this as its supertype is can only have
    /// implemenations within the eagle_vision crate. See
    /// the [sealed traits](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
    /// section of the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines)
    pub trait EagleVisionSealed {}
}

#[cfg(test)] mod test;