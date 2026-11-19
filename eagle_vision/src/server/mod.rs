#![warn(unused)]

use std::collections::VecDeque;

use crate::bevy::prelude::*;
use bevy::platform::collections::HashMap;
use crate::bevy::SystemSet;

use eagle_squak::id::{EagleVisionId, EditorId, LocaleId, SourceId};
use eagle_squak::protocol::{
    ClientHello, ClientRequest, ClientRequestBody, HandshakeError, PROTOCOL_VERSION, RejectionReason, RequestId, RequestRejection, ServerMessage, ServerWelcome, SessionId,
};

use crate::aspect::locale::Locale;
use crate::engine_plugin::GuaranteedMarkerComponent as _;
use crate::engine_plugin::{IdEntityMap, SourceAllocator};
use crate::aspect::editor::{EditorAnonymous, EditorErased};
use crate::aspect::metadata::BaseLocale;
use crate::op::{self, OpCompleted, OpId, OpIdGen};
use crate::schedule::{EngineSet, Update};
use crate::subscription::{SnapshotProduced, SubscriptionDiffProduced, SubscriptionRequestCompleted, SubscriptionRequestId, Subscriptions};

pub mod session;
pub mod dispatch;
pub mod transport;

pub use session::{Session, Sessions, PendingHellos, HandshakeResults};
use session::{accept_connections, SessionCounter};

/// The base server that provides the base abstractions that the language engine speaks.
/// 
/// Transport server plugins sit on top of this layer.
/// 
// The server layer owns sessions and mailboxes.
//
// It dispatches protocol requests into engine work and maps completed/rejected
// engine work back to protocol messages. It does not interpret queries, inspect
// semantic graph components, or construct subscription payloads.
pub struct EagleVisionServerPlugin;

impl Plugin for EagleVisionServerPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<SessionCounter>()
        .init_resource::<Sessions>()
        .init_resource::<PendingHellos>()
        .init_resource::<HandshakeResults>()
        .init_resource::<RequestTracker>()
        .add_message::<IncomingClientRequest>()
        .add_message::<RequestCompleted>()
        .add_message::<OutboundServerMessage>()
        .configure_sets(Update, (
            ServerSet::HandleConnections,
            ServerSet::ReadRequests,
            EngineSet::ExecuteOps,
            EngineSet::UpdateSubscriptions,
            ServerSet::FillOutboxes,
            ServerSet::SendOutboxes,
        ).chain())
        .add_systems(Update, (
            accept_connections.in_set(ServerSet::HandleConnections),
            dispatch::dispatch_requests.in_set(EngineSet::ExecuteOps),
            (
                route_subscription_outputs,
                route_completed_request_messages,
                route_outbound_messages
            ).chain().in_set(ServerSet::FillOutboxes),
        ))
        .add_observer(on_op_completed)
        .add_observer(on_subscription_completed)
        // .add_observer(on_op_rejected)
        ;
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerSet {
    /// The time when Welcome messages are handled.
    HandleConnections,

    /// The time when the server reads and consumes requests
    /// populated by the transport plugin.
    ReadRequests,

    /// The time when outboxes are filled with messages.
    FillOutboxes,

    /// The time when a transport plugin consumes the outbox messages
    /// and actually sends them out of the engine.
    SendOutboxes,
}

/// Per-session outbound, to be drained by the transport sever layer.
#[derive(Debug, Default, Component)]
pub struct Outbox(pub VecDeque<ServerMessage>);

impl Outbox {
    fn push(&mut self, message: ServerMessage) {
        self.0.push_back(message);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrackedId {
    Op(OpId),
    SubscriptionRequest(SubscriptionRequestId),
}

impl From<OpId> for TrackedId {
    fn from(value: OpId) -> Self {
        Self::Op(value)
    }
}

impl From<SubscriptionRequestId> for TrackedId {
    fn from(value: SubscriptionRequestId) -> Self {
        Self::SubscriptionRequest(value)
    }
}

/// Map of incomplete requests to the session + request that spawned it.
#[derive(Debug, Default, Resource)]
pub struct RequestTracker(pub HashMap<TrackedId, (SessionId, RequestId)>);

impl RequestTracker {
    fn track(&mut self, id: TrackedId, sess_id: SessionId, req_id: RequestId) {
        self.0.insert(id, (sess_id, req_id));
    }

    fn get_sub_req(&self, id: SubscriptionRequestId) -> Option<(SessionId, RequestId)> {
        self.0.get(&TrackedId::from(id)).cloned()
    }

    fn track_op(&mut self, id: OpId, sess_id: SessionId, req_id: RequestId) {
        self.track(id.into(), sess_id, req_id);
    }

    fn track_sub_req(&mut self, id: SubscriptionRequestId, sess_id: SessionId, req_id: RequestId) {
        self.track(id.into(), sess_id, req_id);
    }

    fn finish_tracking(&mut self, id: TrackedId) -> Option<(SessionId, RequestId)> {
        self.0.remove(&id)
    }
}

fn on_op_completed(
    event: On<OpCompleted>,
    sessions: Res<Sessions>,
    mut tracked: ResMut<RequestTracker>,
    mut completed_requests: MessageWriter<RequestCompleted>,
) {
    if let Some((sess_id, req_id)) = tracked.finish_tracking(event.op.into()) {
        completed_requests.write(RequestCompleted { session: sessions[sess_id], request: req_id });
    } else {
        // When the server plugin exists, it's the only one allowed to spawn ops.
        bevy::log::debug!("Tried to kill op that's not alive.");
        panic!("Tried to kill op that's not alive.");
    }
}

// Kept for when an op can reject outside of validation.
// fn on_op_rejected(
//     event: On<OpRejected>,
//     sessions: Res<Sessions>,
//     mut live_ops: ResMut<TrackedOps>,
//     mut outbox_query: BevyQuery<&mut Outbox>,
// ) {
//     if let Some((sess_id, req_id)) = live_ops.kill(event.op) {
//         outbox_query.get_mut(sessions[sess_id]).unwrap().push(ServerMessage::RequestRejected {
//             rejection: RequestRejection { id: req_id, reason: event.reason.into() },
//         });
//     }
// }

fn on_subscription_completed(
    event: On<SubscriptionRequestCompleted>,
    sessions: Res<Sessions>,
    mut tracked: ResMut<RequestTracker>,
    mut completed_requests: MessageWriter<RequestCompleted>,
) {
    if let Some((sess_id, req_id)) = tracked.finish_tracking(event.request.into()) {
        completed_requests.write(RequestCompleted { session: sessions[sess_id], request: req_id });
    }
}

impl<'w, 's> Commands<'w, 's> {
    pub fn queue_server_message(&mut self, session: Entity, message: ServerMessage) {
        self.write_message(OutboundServerMessage {
            session,
            message,
        });
    }
}

#[derive(Message)]
pub struct OutboundServerMessage {
    pub session: Entity,
    pub message: ServerMessage,
}

/// All requests that came from clients since the beginning of the last Update loop.
/// Filled by the transport server plugin layer.
#[derive(Debug, Clone, Message)]
pub struct IncomingClientRequest {
    pub session: SessionId,
    pub request: ClientRequest,
}

#[derive(Debug, Clone, Copy, Message)]
pub struct RequestCompleted {
    pub session: Entity,
    pub request: RequestId,
}

fn route_subscription_outputs(
    mut snapshots: MessageReader<SnapshotProduced>,
    mut diffs: MessageReader<SubscriptionDiffProduced>,
    tracked: Res<RequestTracker>,
    sessions: Res<Sessions>,
    mut outboxes: BevyQuery<&mut Outbox>,
) {
    for produced in snapshots.read() {
        if let Some((sess_id, req_id)) = tracked.get_sub_req(produced.id) {
            let session_eid = sessions[sess_id];

            if let Ok(mut outbox) = outboxes.get_mut(session_eid) {
                outbox.push(ServerMessage::Snapshot {
                    id: req_id,
                    snapshot: produced.snapshot.clone(),
                });
            }
        }
    }

    for produced in diffs.read() {
        let session_eid = produced.view.inner();

        if let Ok(mut outbox) = outboxes.get_mut(session_eid) {
            outbox.push(ServerMessage::SubscriptionDiff {
                sub_id: produced.id,
                diff: produced.diff.clone(),
            });
        }
    }
}

fn route_completed_request_messages(
    mut completed_requests: MessageReader<RequestCompleted>,
    mut outbox_query: BevyQuery<&mut Outbox>
) {
    for RequestCompleted { session: sess_eid, request: req_id } in completed_requests.read().copied() {
        if let Ok(mut outbox) = outbox_query.get_mut(sess_eid) {
            outbox.push(ServerMessage::CommandComplete { id: req_id })
        }
    }
}

fn route_outbound_messages(
    mut messages: MessageReader<OutboundServerMessage>,
    mut outboxes: BevyQuery<&mut Outbox>,
) {
    for outbound in messages.read() {
        if let Ok(mut outbox) = outboxes.get_mut(outbound.session) {
            outbox.push(outbound.message.clone());
        }
    }
}