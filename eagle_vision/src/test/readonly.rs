use std::collections::HashSet;

use super::*;

#[test]
fn readonly_client_is_rejected_and_gets_no_source() {
    let mut settings = NewProjectSettings::new();
    settings.set_project_name("Original");

    let mut server = TestServer::with_hello(settings, ClientHello {
        username: None,
        locales: vec!["en-US".into()],
        flags: HashSet::from([ClientHelloFlags::Readonly]),
        protocol_version: PROTOCOL_VERSION,
    }).unwrap();

    assert!(server.welcome.allocated_source.is_none());

    let req_id = server.request(ClientRequestBody::SetProjectName { name: "New Name".into() });

    let messages = server.drain();
    assert!(messages.iter().any(|m| matches!(
        m, ServerMessage::RequestRejected {
            rejection: RequestRejection {
                id: req,
                reason: RejectionReason::WriteCommandInReadonlyMode,
            }
        }
    )));
    assert!(!messages.iter().any(|m| matches!(m, ServerMessage::CommandComplete { .. })));
}