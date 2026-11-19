//! Communication between the Eagle Vision server and its clients

pub mod aspect;
pub mod id;
pub mod id_allocator;
pub mod subscription;
pub mod protocol;

/// A reexport of all the specific AspectedIds from the [`id`] module.
pub mod aspect_ids {
    pub use crate::id::{
        ExecutableId,
        FunctionId,
        ExpressionId,
        TyId,
        TextId,
        LocaleId,
        ParameterId,
        EditorId,
    };
}