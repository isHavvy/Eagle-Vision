use bevy::prelude::*;
use smallvec::smallvec;

use crate::diagnostic::{self, Diagnostic, DiagnosticVariant, Diagnostics};

#[derive(Debug, Component)]
pub struct Locale(#[doc = "Code"] pub String);

impl Locale {
    #[inline]
    pub fn code(&self) -> &str {
        &self.0
    }
}

fn find_locale_with_code(
    locale_query: &Query<(Entity, &Locale)>,
    code: &str,
) -> Option<Entity> {
    locale_query.iter()
    .find(|&(_, locale)| locale.code() == code)
    .map(|(entity, _locale)| entity)
}

#[deprecated(note = "We should be doing diagnostics checks in a separate dedicated system, future work to rewire")]
pub(crate) fn insert_locale_system(
    code: In<String>,

    mut commands: Commands,
    locale_query: Query<(Entity, &Locale)>,
    mut diagnostics_query: Query<&mut Diagnostics>,
    mut diagnostic_query: Query<(Entity, &mut Diagnostic)>,
) -> Entity {
    let new_locale_entity = commands.spawn_empty().id();

    // Check if there's already a locale with the same code.
    // If there is, check if there's a DuplicateLocales diagnostic on it.
    // If there is
    //      there are more duplicates already, so just
    //      add this locale to that diagnostic and this diagnostic to this locale's diagnostics.
    // Otherwise,
    //      there are not more duplicates, so
    //      create a new duplicate locales diagnostic for the two locales. 
    let duplicate_locale = find_locale_with_code(&locale_query.as_readonly(), &code.0);

    if let Some(duplicate_locale_entity) = duplicate_locale {
        let existing_duplicate_locales_diagnostic = diagnostic::try_get_single_diagnostic_mut(
            &diagnostics_query.as_readonly(),
            &diagnostic_query.as_readonly(),
            duplicate_locale_entity,
            DiagnosticVariant::DuplicateLocales
        );

        match existing_duplicate_locales_diagnostic {
            Some(diagnostic_entity) => {
                commands.entity(new_locale_entity).insert(Diagnostics(vec![diagnostic_entity]));
                diagnostic_query.get_mut(diagnostic_entity).unwrap().1.add_entity(new_locale_entity);
            },
            None => {
                let new_diagnostic = commands.spawn(Diagnostic::DuplicateLocales(smallvec![duplicate_locale_entity, new_locale_entity])).id();
                commands.entity(new_locale_entity).insert(Diagnostics(vec![new_diagnostic]));
                
                match diagnostics_query.get_mut(duplicate_locale_entity) {
                    Ok(mut diagnostics) => { diagnostics.push(new_diagnostic); },
                    Err(..) => { commands.entity(duplicate_locale_entity).insert(Diagnostics(vec![new_diagnostic])); },
                }
            },
        }
    }

    commands.entity(new_locale_entity).insert(Locale(code.0));

    new_locale_entity
}

#[deprecated]
#[expect(unused_mut, unused_variables)]
pub(crate) fn update_locale_system(
    input: In<(Entity, String)>,

    mut commands: Commands,
    mut locale_query: Query<(Entity, &mut Locale)>,
    mut diagnostics_query: Query<&mut Diagnostics>,
    mut diagnostic_query: Query<(Entity, &mut Diagnostic)>,
) {
    let (locale_entity, updated_code) = input.0;

    // Do nothing if the locale's code is already the updated code.
    if locale_query.get(locale_entity).unwrap().1.code() == updated_code {
        return;
    }

    let duplicate_locale_diagnostic = diagnostic::try_get_single_diagnostic_mut(
        &diagnostics_query.as_readonly(),
        &diagnostic_query.as_readonly(),
        locale_entity,
        DiagnosticVariant::DuplicateLocales
    );

    if let Some(diagnostic) = duplicate_locale_diagnostic {
        // remove the diagnostic from this entity, possibly despawning it altogether, which would remove it from the other locale as well 
    }

    // Check for new duplicates and create/update a diagnostic for duplicateness.
    // This one differs from above because this entity might already have a diagnostics component.

    locale_query.get_mut(locale_entity).unwrap().1.0 = updated_code;
}

#[cfg(all(false, test))] // Move to a diagnostics plugin
mod test {
    use std::assert_matches;

    use crate::{diagnostic::DiagnosticVariant, eaglevision::EagleVision};

    #[test]
    pub fn duplicate_locales() {
        let mut ev = EagleVision::new(Default::default());
        let locale = ev.insert_locale("en-US".to_string());

        let diagnostics = ev.diagnostics_of(locale);
        let duplicate_locale = diagnostics.try_get_one(DiagnosticVariant::DuplicateLocales).unwrap();

        assert_eq!(duplicate_locale.entity_len(), 2);
    }

    #[test]
    pub fn non_duplicate_locales() {
        let mut ev = EagleVision::new(Default::default());
        let locale = ev.insert_locale("fake-FAKE".to_string());

        let diagnostics = ev.diagnostics_of(locale);
        assert_matches!(diagnostics.try_get_one(DiagnosticVariant::DuplicateLocales), None);
    }
}