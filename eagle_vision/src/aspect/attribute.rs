use eagle_squak::id::{AttributeDefinitionId, EagleVisionId};

use crate::bevy::prelude::*;

#[derive(Debug, Clone, Copy, Component)]
pub struct AttributeKind {
    // #TODO(Havvy) pub applies_to: AspectFilter
}

#[derive(Debug, Clone, Copy, Component)]
pub struct Attribute {
    pub kind: AttributeDefinitionId,
    pub on: EagleVisionId,
}