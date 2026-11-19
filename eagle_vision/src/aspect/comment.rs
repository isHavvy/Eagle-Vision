//! Comments: annotation-layer nodes attached to semantic nodes.

use bevy::prelude::*;

use eagle_squak::id::{EagleVisionId, TextId};

/// The comment itself. Entity also carries `CommentId`.
#[derive(Debug, Component)]
pub struct Comment {
    pub body: TextId,
}

/// What the comment annotates. Untyped on purpose: documentation
/// targets functions, explanations target expressions, safety
/// justifications may target whatever unsafe ends up being.
#[derive(Debug, Component)]
pub struct CommentOn(pub EagleVisionId);