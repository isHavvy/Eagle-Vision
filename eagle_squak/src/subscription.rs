use crate::aspect::AspectId;
use crate::id::EagleVisionId;

/// An id for subscribing to information
/// 
/// The client provides which ID to use for each subscription with the subscribe command.
/// 
/// When 
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash,)]
pub struct SubscriptionId(pub u32);

/// Kinds of info a client can subscribe to
#[derive(Debug, Clone,)]
pub enum SubscriptionTarget {
    /// Watch a single specific aspect
    Aspect(EagleVisionId),

    /// Watch all items matching a specific  [`Aspect`]
    AllOfAspect(AspectId),

    /// Watch an aspect and what it thinks it children are recursively.
    Children(EagleVisionId),

    /// Watch the project name
    /// 
    /// Clients may want to watch this separately from other project metadata if the name
    /// is displayed to the user such as in a window title.
    ProjectName,

    /// Watch all metadata on the project
    Metadata,
}