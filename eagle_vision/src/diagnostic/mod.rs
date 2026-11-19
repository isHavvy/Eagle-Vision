use std::ops::{Deref, DerefMut};

use bevy::prelude::*;
use smallvec::SmallVec;

/// Potential issue in the project
#[derive(Debug, Clone, Component)]
#[non_exhaustive]
pub enum Diagnostic {
    /// Multiple locales that have the same code
    DuplicateLocales(SmallVec<[Entity; 2]>)
}

impl Diagnostic {
    pub fn is_variant(&self, variant: DiagnosticVariant) -> bool {
        #[expect(unreachable_patterns)]
        match (self, variant) {
            (Diagnostic::DuplicateLocales(..), DiagnosticVariant::DuplicateLocales) => true,

            _ => false
        }
    }

    /// Number of entities this diagnostic affects.
    pub fn entity_len(&self) -> usize {
        match self {
            Diagnostic::DuplicateLocales(locales) => locales.len(),
        }
    }

    pub(crate) fn add_entity(&mut self, entity: Entity) {
        #[expect(unreachable_patterns)]
        match self {
            Diagnostic::DuplicateLocales(entities) => { entities.push(entity); },

            _ => panic!("Tried to push an entity onto a diagnostic that doesn't take multiple entities.")
        }
    }
}

/// The variants of [`Diagnostic`] without their associated data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DiagnosticVariant {
    DuplicateLocales,
}

/// List of diagnostics associated with the entity.
#[derive(Debug, Default, Component)]
pub struct Diagnostics(pub Vec<Entity>);

impl Deref for Diagnostics {
    type Target = Vec<Entity>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Diagnostics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct DiagnosticsOf(pub Vec<Diagnostic>);

impl DiagnosticsOf {
    pub fn try_get_one(&self, variant: DiagnosticVariant) -> Option<&Diagnostic> {
        self.0.iter().find(|diagnostic| diagnostic.is_variant(variant))
    }
}

pub(crate) fn try_get_single_diagnostic_mut(
    diagnostics_query: &Query<&Diagnostics>,
    diagnostic_query: &Query<(Entity, &Diagnostic)>,
    thing_entity: Entity,
    diagnostic_variant: DiagnosticVariant,
) -> Option<Entity> {
    let diagnostics = match diagnostics_query.get(thing_entity) {
        Ok(diagnostics) => diagnostics.0.iter(),
        Err(_) => { return None; },
    };

    for (entity, diagnostic) in diagnostic_query.iter_many(diagnostics).matched() {
        if diagnostic.is_variant(diagnostic_variant) {
            return Some(entity);
        }
    }

    None
}