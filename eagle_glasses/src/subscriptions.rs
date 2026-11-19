use bevy::prelude::*;
use eagle_squak::{id::{EagleVisionId, LocaleId}, protocol::subscription::{AspectData, SubscriptionItem}};

#[derive(Resource, Default)]
pub struct ProjectNameCache {
    pub id: Option<EagleVisionId>,
    pub locale: Option<LocaleId>,
    pub value: Option<String>,
}

impl ProjectNameCache {
    pub fn display_name(&self) -> &str {
        self.value.as_deref().unwrap_or("Untitled Project")
    }

    pub fn apply_item(&mut self, item: SubscriptionItem) {
        let AspectData::Text(text) = item.data;

        self.id = Some(item.id);
        self.locale = Some(text.locale);
        self.value = Some(text.value);
    }

    pub fn remove(&mut self, id: EagleVisionId) {
        if self.id == Some(id) {
            *self = Self::default();
        }
    }
}