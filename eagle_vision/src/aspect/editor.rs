use bevy::prelude::*;
use eagle_squak::id::TextId;

use crate::private::EagleVisionSealed;
use crate::eaglevision::GuaranteedMarkerComponent;

#[derive(Debug, Component)]
pub struct Editor {
    pub username: TextId,
}

/// Marker component for the anonymous editor which
/// claims edits from anonymous clients.
#[derive(Debug, Component)]
pub struct EditorAnonymous;

impl EagleVisionSealed for EditorAnonymous {}
impl GuaranteedMarkerComponent for EditorAnonymous {}

/// Marker component for the anonymous editor which exists
/// but was not transferred into the project when the
/// dependency was pulled in.
#[derive(Debug, Component)]
pub struct EditorErased;

impl EagleVisionSealed for EditorErased {}
impl GuaranteedMarkerComponent for EditorErased {}