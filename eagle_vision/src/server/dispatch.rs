use crate::{bevy::prelude::*};
use crate::engine_plugin::{View, ViewEntity};
use crate::subscription::{SubscriptionRequestIdGen};
use super::*;

struct ValidateAndEmitBody<'a, 'w, 's, Op: op::UnvalidatedOp> {
    op: Op,
    ids: &'a IdEntityMap,
    commands: &'a mut Commands<'w, 's>,
    outbox: &'a mut Outbox,
    tracked: &'a mut RequestTracker,
    source_id: SourceId,
    sess_id: SessionId,
    req_id: RequestId,
    op_id: OpId,
}

fn validate_and_emit<Op: op::UnvalidatedOp>(args: ValidateAndEmitBody<Op>) {
    match args.op.check(args.ids, args.source_id) {
        Ok(op_event) => {
            args.tracked.track_op(args.op_id, args.sess_id, args.req_id);
            args.commands.trigger(op_event);
        },

        Err(reason) => {
            args.outbox.push(ServerMessage::RequestRejected {
                rejection: RequestRejection { id: args.req_id, reason: reason.into() }
            });
        }
    }
}

pub(super) fn dispatch_requests(
    mut commands: Commands,
    mut inbox: MessageReader<IncomingClientRequest>,
    sessions: Res<Sessions>,
    mut session_query: BevyQuery<(&Session, &View, &mut Outbox)>,
    mut op_id_gen: ResMut<OpIdGen>,
    mut sub_req_id_gen: ResMut<SubscriptionRequestIdGen>,
    mut tracker: ResMut<RequestTracker>,
    ids: Res<IdEntityMap>,
) {
    for IncomingClientRequest { session: sess_id, request: ClientRequest { id: req_id, body }} in inbox.read().cloned() {
        let session_eid = sessions[sess_id];
        let (session, view, mut outbox) = session_query.get_mut(session_eid).unwrap();
        let editor = session.editor;

        // #[TODO(Havvy)]: Layering issue: Ops should be passed in an entity that has the view data.
        let preferred_locale = view.first_locale();

        // Reject mutating commands for readonly clients.
        if session.readonly && !body.is_readonly() {
            outbox.push(ServerMessage::RequestRejected {
                rejection: RequestRejection { id: req_id, reason: RejectionReason::WriteCommandInReadonlyMode },
            });
            continue;
        }

        // Make the ids eagerly. If not used, it's only a tiny inefficiency.
        let op_id = op_id_gen.next();
        let sub_req_id = sub_req_id_gen.next();

        match body {
            // #[Section(Begin)] Query Requests
            // #[Safety(ViewEntity::new_unchecked)]: The session entities are always initialized with View components.

            ClientRequestBody::Snapshot { query } => {
                tracker.track_sub_req(sub_req_id, sess_id, req_id);

                commands.trigger(crate::subscription::Snapshot {
                    id: sub_req_id,
                    view: ViewEntity::new_unchecked(session_eid),
                    query,
                });
            },

            ClientRequestBody::Subscribe { id, query } => {
                tracker.track_sub_req(sub_req_id, sess_id, req_id);

                commands.trigger(crate::subscription::Subscribe {
                    id: sub_req_id,
                    view: ViewEntity::new_unchecked(session_eid),
                    sub_id: id,
                    query,
                });
            },

            ClientRequestBody::Unsubscribe { id } => {
                tracker.track_sub_req(sub_req_id, sess_id, req_id);

                commands.trigger(crate::subscription::Unsubscribe {
                    id: sub_req_id,
                    view: ViewEntity::new_unchecked(session_eid),
                    sub_id: id,
                });
            },

            // #[Section(End)]: Query Requests

            // #[Section(Begin)]: Op requests
            // #[SAFETY(UNWRAP)] session.source is guaranteed to be Option::Some by this point.
            //                   it is only None when the client is readonly, but readonly clients
            //                   are rejected for write ops earlier in this function.
            ClientRequestBody::SetProjectName { name: new_name } => {
                tracker.track_op(op_id, sess_id, req_id);
                commands.trigger(op::set_project_name::SetProjectName {
                    op: op_id,
                    name: new_name,
                    editor,
                    locale: preferred_locale,
                });
            }

            ClientRequestBody::CreateAttributeDefinition { id } => {
                validate_and_emit(ValidateAndEmitBody {
                    op: op::create_tag::CreateAttributeDefinitionUnvalidated {
                        op: op_id,
                        id, editor
                    },
                    ids: &*ids,
                    commands: &mut commands,
                    outbox: &mut outbox,
                    tracked: &mut tracker,
                    source_id: session.source.unwrap(),
                    sess_id, req_id, op_id,
                });
            }

            ClientRequestBody::CreateComment { comment, body, on } => {
                validate_and_emit(ValidateAndEmitBody {
                    op: op::create_comment::CreateCommentUnvalidated {
                        op: op_id,
                        comment, body, on, editor,
                    },
                    ids: &*ids,
                    commands: &mut commands,
                    outbox: &mut outbox,
                    tracked: &mut tracker,
                    source_id: session.source.unwrap(),
                    sess_id, req_id, op_id,
                });
            }

            ClientRequestBody::CreateAttribute { id, on, definition } => {
                validate_and_emit(ValidateAndEmitBody {
                    op: op::apply_tag::CreateAttributeUnvalidated {
                        op: op_id,
                        id, on, definition, editor,
                    },
                    ids: &*ids,
                    commands: &mut commands,
                    outbox: &mut outbox,
                    tracked: &mut tracker,
                    source_id: session.source.unwrap(),
                    sess_id, req_id, op_id,
                });
            }
            ClientRequestBody::CreateText { text: _text } => {
                outbox.push(ServerMessage::RequestRejected {
                    rejection: RequestRejection { id: req_id, reason: RejectionReason::CommandUnimplemented },
                });
            },

            // #[Section(End)]: Op requests

            _ => {
                outbox.push(ServerMessage::RequestRejected {
                    rejection: RequestRejection { id: req_id, reason: RejectionReason::CommandUnimplemented },
                });
            }
        }
    }
}