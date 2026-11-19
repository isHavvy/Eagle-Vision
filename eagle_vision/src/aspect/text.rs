use bevy::ecs::component::Component;
use smallvec::{SmallVec, smallvec};

use eagle_squak::id::{LocaleId, TextId};

// #[TODO(Havvy)]: Determine whether or not Text aspects can have zero values.
#[derive(Debug, Component)]
pub struct Text {
    texts: SmallVec<[(LocaleId, String); 1]>
}

impl Text {
    pub fn new(base_locale: LocaleId, value: impl Into<String>) -> Self {
        Text {
            texts: smallvec![(base_locale, value.into())]
        }
    }

    pub fn new_empty() -> Self {
        Text {
            texts: smallvec![]
        }
    }

    pub fn get_value(&self, locale: LocaleId) -> Option<&str> {
        self.texts
        .iter()
        .find(|(id, _value)| *id == locale)
        .map(|(_locale, value)| value.as_str())
    }

    pub fn upsert_value(&mut self, locale: LocaleId, value: String) {
        let existing_value = self.texts
        .iter_mut()
        .find(|(id, _value)| *id == locale);

        match existing_value {
            Some(&mut (_locale, ref mut old_value)) => {
                *old_value = value;
            },
            None => {
                self.texts.push((locale, value));
            },
        }
    }

    pub fn select_text_value<'a>(&self, mut preferred_locales: impl Iterator<Item=LocaleId>) -> Option<(LocaleId, &str)> {
        preferred_locales.find_map(|locale| {
            self.get_value(locale).map(|value| (locale, value))
        }).or_else(|| {
            self.texts.iter()
            .next()
            .map(|(locale, value)| (*locale, value.as_str()))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(crate) struct AspectName(pub TextId);

#[cfg(test)]
mod test {
    use eagle_squak::id::{TextId, LocaleId};
    use crate::aspect::{locale::Locale, text::Text};

    #[test]
    fn text_stuff() {
        let en = LocaleId::new(0, 0);
        let es = LocaleId::new(0, 1);

        let mut hello = Text::new(en, "hello");

        assert!(hello.get_value(en) == Some("hello"));
        assert!(hello.get_value(es).is_none());

        hello.upsert_value(en, "hi".to_string());

        assert!(hello.get_value(en) == Some("hi"));
        assert!(hello.get_value(es).is_none());

        hello.upsert_value(es, "hola".to_string());

        assert!(hello.get_value(en) == Some("hi"));
        assert!(hello.get_value(es) == Some("hola"));
    }
}