use crate::id::{EagleVisionId, LocaleId};

#[derive(Debug, Clone)]
pub struct SubscriptionItem {
    pub id: EagleVisionId,
    pub data: AspectData,
}

#[derive(Debug, Clone)]
pub enum AspectData {
    Text(TextSnapshot),
}

#[derive(Debug, Clone)]
pub struct TextSnapshot {
    /// The locale actually selected for this client.
    pub locale: LocaleId,

    /// The localized value selected for this client.
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Diff {
    pub added: Vec<SubscriptionItem>,
    pub updated: Vec<SubscriptionItem>,
    pub removed: Vec<EagleVisionId>,
}