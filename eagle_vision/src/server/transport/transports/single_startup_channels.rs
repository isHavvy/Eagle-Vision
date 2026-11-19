//! Single client server transport over mpsc channels that connects at startup.
//! 
//! This is intended to be used for Rust programs that start
//! the language engine in the same process with that program
//! being the client.

use std::sync::mpsc::{Receiver, Sender};

use crate::bevy::prelude::*;
use crate::server::IncomingClientRequest;
use eagle_squak::protocol::{ClientRequest, ServerMessage, SessionId};

use crate::schedule::Update;
use crate::server::{Outbox, Sessions, ServerSet, transport::TransportPlugin};

#[derive(Debug)]
pub struct SingleClientPlugin;

#[derive(Debug)]
pub struct SingleClientStartupData {
    pub inbound: Receiver<ClientRequest>,
    pub outbound: Sender<ServerMessage>,
}

#[derive(Debug)]
pub struct SingleClientHandle {
    pub requests: Sender<ClientRequest>,
    pub messages: Receiver<ServerMessage>,
}

impl Plugin for SingleClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (
                read_requests.in_set(ServerSet::ReadRequests),
                send_messages.in_set(ServerSet::SendOutboxes),
            ))
        ;
    }
}

impl TransportPlugin for SingleClientPlugin {
    type StartupData = SingleClientStartupData;

    fn install_startup_data(
        app: &mut App,
        startup: Self::StartupData,
    ) {
        app.insert_non_send(InboundRequests(startup.inbound));
        app.insert_non_send(OutboundMessages(startup.outbound));
    }
}

impl SingleClientPlugin {
    const SESSION_ID: SessionId = SessionId(1);
}

#[derive(Debug)]
struct InboundRequests(Receiver<ClientRequest>);

#[derive(Debug)]
struct OutboundMessages(Sender<ServerMessage>);

fn read_requests(
    receiver: NonSendMut<InboundRequests>,
    mut server_inbox: MessageWriter<IncomingClientRequest>,
) {
    while let Ok(request) = receiver.0.try_recv() {
        server_inbox.write(IncomingClientRequest {
            session: SingleClientPlugin::SESSION_ID,
            request,
        });
    }
}

fn send_messages(
    sessions: Res<Sessions>,
    mut outbox_query: BevyQuery<&mut Outbox>,
    sender: NonSendMut<OutboundMessages>,
) {
    let session_eid = sessions[SingleClientPlugin::SESSION_ID];

    if let Ok(mut outbox) = outbox_query.get_mut(session_eid) {
        while let Some(msg) = outbox.0.pop_front() {
            let _ = sender.0.send(msg);
        }
    }
}