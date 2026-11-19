use crate::aspect::AspectId;
use crate::id::EagleVisionId;

/// Query for aspect data
#[derive(Debug, Clone, Copy)]
pub enum Query {
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