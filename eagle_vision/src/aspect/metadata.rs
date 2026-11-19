//! Data about the project itself

use bevy::prelude::*;

use crate::private::EagleVisionSealed;
use crate::eaglevision::GuaranteedMarkerComponent;

/// Marker component for a Text aspect that is the project name.
/// 
/// Unnamed projects will be missing this component.
#[derive(Debug, Component)]
pub struct ProjectName;

/// This marker component is put on the Locale that
/// is the default locale of the project. If a client's
/// locales do not match any for a Text entry, we return
/// the base locale value if it exists. Otherwise we
/// return an arbitrary (as far as the client is concerned)
/// locale's value.
#[derive(Debug, Component)]
pub struct BaseLocale;

impl EagleVisionSealed for BaseLocale {}
impl GuaranteedMarkerComponent for BaseLocale {}

/// Editors that created the project
/// 
/// This marker component is attached to the [`EditorProfile`](`super::editor::EditorProfile`)s
/// that created this project.
#[derive(Debug, Component)]
pub struct Creator;