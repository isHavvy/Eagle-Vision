#![allow(unused)]

mod bevy;
pub mod engine;
pub mod engine_plugin;
pub mod bytecode;
pub mod eaglevision;
pub mod diagnostic;
pub mod aspect;
pub mod server;
pub mod init;
pub mod subscription;
pub mod op;
pub mod schedule;

mod private {
    /// Trait only implementable in the eagle_vision crate.
    /// 
    /// A trait that has this as its supertype is can only have
    /// implemenations within the eagle_vision crate. See
    /// the [sealed traits](https://rust-lang.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed)
    /// section of the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines)
    pub trait EagleVisionSealed {}
}

// #[TODO(Havvy)]: Get tests working with the engine::EagleVision version.
//                 This is going to require being able to tell `hello` to run.
#[cfg(all(test, false))] mod test;