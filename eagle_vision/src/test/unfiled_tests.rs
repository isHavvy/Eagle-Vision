//! Tests that don't have a good named file

use super::*;
use crate::aspect::metadata::ProjectName;
use crate::aspect::text::Text;
use crate::init::NewProjectSettings;

#[test]
fn subscribe_then_edit_project_name() {
    let mut settings = NewProjectSettings::new();
    settings.set_project_name("Original");

    let mut server = TestServer::new(settings);

    assert!(server.welcome.allocated_source.is_some());
    assert!(!server.welcome.locales.is_empty());

    // Subscribe: snapshot must precede completion.
    let (sub, req) = server.subscribe(Query::ProjectName);
    let messages = server.drain();
    let snap = position(&messages, |m| matches!(m, ServerMessage::Snapshot { sub_id, .. } if *sub_id == sub));
    let done = position(&messages, |m| matches!(m, ServerMessage::CommandComplete { revision } if *revision == req));
    assert!(snap < done, "snapshot before completion: {messages:#?}");

    // Edit: update must precede completion.
    let req = server.request(ClientRequestBody::SetProjectName { name: "Renamed".into() });
    let msgs = server.drain();
    let upd = position(&msgs, |m| matches!(m, ServerMessage::SubscriptionUpdate { sub_id, .. } if *sub_id == sub));
    let done = position(&msgs, |m| matches!(m, ServerMessage::CommandComplete { revision } if *revision == req));
    assert!(upd < done, "update before completion: {msgs:#?}");

    match &msgs[upd] {
        ServerMessage::SubscriptionUpdate { added, updated, removed, .. } => {
            assert!(removed.is_empty());
            assert!(added.is_empty());
            assert_eq!(updated.len(), 1);
        }
        _ => unreachable!(),
    }

    // World agrees with the message.
    let base = server.base_locale();
    let world = server.ev.world();
    let mut q = world.try_query_filtered::<&Text, With<ProjectName>>().unwrap();
    assert_eq!(q.single(world).unwrap().get_value(base), Some("Renamed"));
}

fn position(msgs: &[ServerMessage], pred: impl Fn(&ServerMessage) -> bool) -> usize {
    msgs.iter().position(pred).expect("expected message missing")
}