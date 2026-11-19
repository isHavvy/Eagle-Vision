#![warn(unused)]

use crate::engine_plugin::{View, ViewEntity};
use crate::schedule::{EngineSet, Update};
use crate::{aspect::text::Text, bevy::prelude::*};
use crate::aspect::metadata::ProjectName;
use bevy::platform::collections::HashMap;
use eagle_squak::protocol::{Snapshot as SnapshotData};
use eagle_squak::protocol::subscription::{AspectData, Diff, TextSnapshot};
use eagle_squak::{id::EagleVisionId, protocol::{SubscriptionId, subscription::SubscriptionItem}, query::Query};
use smallvec::SmallVec;

/// The Subscription Layer
/// 
/// The subscription layer owns query interpretation and view maintenance.
///
/// It consumes subscription request events, reads the semantic graph, and emits
/// snapshot/update messages into session outboxes. It signals request completion
/// only after the corresponding messages have been queued.
#[derive(Debug)]
pub struct SubscriptionPlugin;

impl Plugin for SubscriptionPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<SubscriptionRequestIdGen>()
        .add_message::<SnapshotProduced>()
        .add_message::<SubscriptionDiffProduced>()
        .add_observer(on_subscribe)
        .add_observer(on_unsubscribe)
        .add_observer(on_snapshot)
        .add_systems(Update, handle_snapshots_and_subscriptions.in_set(EngineSet::UpdateSubscriptions))
        ;
    }
}

/// Active subscriptions.
/// 
/// Only placed on entities with a View component.
#[derive(Debug, Clone, Default, Component)]
pub struct Subscriptions {
    pub queries: HashMap<SubscriptionId, Query>,
}

/// Snapshots that need to be created and sent.
/// 
/// Only placed on entities with a View component.
#[derive(Debug, Default, Component)]
pub struct SnapshotRequests {
    pub queue: SmallVec<[SnapshotRequest; 1]>,
}

impl SnapshotRequests {
    fn take(&mut self) -> SmallVec<[SnapshotRequest; 1]> {
        std::mem::take(&mut self.queue)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SnapshotRequest {
    pub id: SubscriptionRequestId,
    pub sub_id: Option<SubscriptionId>,
    pub query: Query,
}

#[derive(Message)]
pub struct SnapshotProduced {
    pub id: SubscriptionRequestId,
    pub snapshot: eagle_squak::protocol::Snapshot,
}

#[derive(Message)]
pub struct SubscriptionDiffProduced {
    pub view: ViewEntity,
    pub id: SubscriptionId,
    pub diff: eagle_squak::protocol::subscription::Diff,
}

fn build_snapshot(world: &mut World, view: &View, query: Query) -> Vec<SubscriptionItem> {    
    match query {
        Query::ProjectName => {
            let locales = view.locales.clone();

            let mut text_query = world.query_filtered::<(&EagleVisionId, &Text), With<ProjectName>>();

            if let Ok((id, text)) = text_query.single(world) {
                // SAFETY(UNWRAP): The project name text entity will always have at least value.
                let (locale, value) = text.select_text_value(locales.iter().copied()).unwrap();

                vec![SubscriptionItem {
                    id: *id,
                    data: AspectData::Text(TextSnapshot {
                        locale,
                        value: value.to_string(),
                    }),
                }]
            } else {
                vec![]
            }
        },

        _ => todo!(),
    }
}

pub fn matches_target(world: &mut World, query: Query, id: EagleVisionId) -> bool {
    match query {
        Query::ProjectName => {
            let mut q = world.query_filtered::<&EagleVisionId, With<ProjectName>>();
            q.iter(world).any(|i| *i == id)
        }
        _ => todo!(),
    }
}

#[derive(Message)]
pub struct SubscriptionChanged {
    pub entity: Entity,
    pub sub_id: SubscriptionId,
    pub kind: SubscriptionChangeKind,
}

pub enum SubscriptionChangeKind {
    Add,
    Remove,
}

fn build_item_for_id(
    world: &mut World,
    view: &View,
    query: Query,
    id: EagleVisionId,
) -> Option<SubscriptionItem> {
    match query {
        Query::ProjectName => {
            let locales = view.locales.clone();

            let mut project_name_query = world.query_filtered::<(&EagleVisionId, &Text), With<ProjectName>>();

            project_name_query.iter(world)
                .find(|(aspect_id, _)| **aspect_id == id)
                .and_then(|(id, text)| {
                    let (locale, value) = text.select_text_value(locales.iter().copied())?;

                    Some(SubscriptionItem {
                        id: *id,
                        data: AspectData::Text(TextSnapshot {
                            locale,
                            value: value.to_string(),
                        }),
                    })
                })
        }

        _ => todo!(),
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct SubscriptionRequestId(pub u64);

#[derive(Debug, Default, Resource)]
pub struct SubscriptionRequestIdGen(u64);

impl SubscriptionRequestIdGen {
    pub fn next(&mut self) -> SubscriptionRequestId {
        let id = SubscriptionRequestId(self.0);
        self.0 += 1;
        id
    }
}

#[derive(Debug, Event)]
pub(crate) struct Subscribe {
    pub id: SubscriptionRequestId,
    pub sub_id: SubscriptionId,
    pub view: ViewEntity,
    pub query: Query,
}

#[derive(Debug, Event)]
pub(crate) struct Unsubscribe {
    pub id: SubscriptionRequestId,
    pub view: ViewEntity,
    pub sub_id: SubscriptionId,
}

#[derive(Debug, Event)]
pub(crate) struct Snapshot {
    pub id: SubscriptionRequestId,
    pub view: ViewEntity,
    pub query: Query,
}

#[derive(Debug, Event)]
pub struct SubscriptionRequestCompleted {
    pub request: SubscriptionRequestId,
}

#[derive(Debug, Event)]
pub struct SubscriptionRequestRejected {
    pub request: SubscriptionRequestId,
    pub reason: SubscriptionRejectionReason,
}

#[derive(Debug, Clone, Copy)]
pub enum SubscriptionRejectionReason {
    DuplicateSubscription(SubscriptionId),
    UnknownSubscription(SubscriptionId),
    UnsupportedQuery(Query),
}

// #[TODO(Havvy)]: This should be two systems, not one.
// #[TODO(Havvy)]: build_snapshot/build_item_for_id needs World.
//                 I'd like to have standard system parameters.
//                 Figure out how to do so.
fn handle_snapshots_and_subscriptions(world: &mut World) {
    let requests: Vec<_> = world.query::<(&mut SnapshotRequests, &View)>()
    .iter_mut(world)
    .map(|(mut requests, view)| (requests.take(), view.clone()))
    .collect();

    for (requests, view) in requests {
        for request in requests {
            let items = build_snapshot(world, &view, request.query);

            world.write_message(SnapshotProduced {
                id: request.id,
                snapshot: SnapshotData {
                    sub_id: request.sub_id,
                    items,
                },
            });

            world.trigger(SubscriptionRequestCompleted {
                request: request.id,
            });
        }
    }

    // Diffs for changed text.
    let changed: Vec<EagleVisionId> = world.query_filtered::<&EagleVisionId, Changed<Text>>()
        .iter(world).copied().collect();
    if changed.is_empty() { return; }

    let subscriptions: Vec<_> = world.query::<(Entity, &View, &Subscriptions)>()
    .iter(world)
    .map(|(eid, a, b)| (eid, a.clone(), b.clone()))
    .collect();

    for (eid, view, subscriptions) in subscriptions {
        for (sub_id, query) in subscriptions.queries.iter() {
            let query = query.clone();
            let updated: Vec<SubscriptionItem> = changed
                .iter()
                .filter(|id| matches_target(world, query, **id))
                .collect::<Vec<_>>()
                .into_iter()
                .filter_map(|id| build_item_for_id(world, &view, query, *id))
                .collect();

            if updated.is_empty() {
                continue;
            }

            world.write_message(SubscriptionDiffProduced {
                // #[Safety(ViewEntity::new_unchecked)]: The entity is from a query of entities with View components.
                view: ViewEntity::new_unchecked(eid),
                id: *sub_id,
                diff: Diff {
                    added: vec![],
                    updated,
                    removed: vec![]
                },
            });
        }
    }
}

// fn on_subscribe(
//     event: On<Subscribe>,
//     mut sessions: BevyQuery<&mut Subscriptions>,
//     mut commands: Commands,
// ) {
//     let mut subscriptions = sessions.get_mut(event.session).unwrap();

//     if subscriptions.queries.contains_key(&event.id) {
//         commands.trigger(SubscriptionRequestRejected {
//             request: event.request,
//             reason: SubscriptionRejectionReason::DuplicateSubscription(event.id),
//         });
//         return;
//     }

//     subscriptions.queries.insert(event.id, event.query);
//     subscriptions.snapshot_requests.push(SnapshotRequest {
//         request: event.request,
//         sub_id: Some(event.id),
//     });
// }

fn on_subscribe(
    event: On<Subscribe>,
    mut views: BevyQuery<(&mut Subscriptions, &mut SnapshotRequests)>,
    mut commands: Commands,
) {
    let (mut subscriptions, mut requests) = views.get_mut(event.view.inner()).unwrap();

    if subscriptions.queries.contains_key(&event.sub_id) {
        commands.trigger(SubscriptionRequestRejected {
            request: event.id,
            reason: SubscriptionRejectionReason::DuplicateSubscription(event.sub_id),
        });
        return;
    }

    subscriptions.queries.insert(event.sub_id, event.query);

    requests.queue.push(SnapshotRequest {
        id: event.id,
        sub_id: Some(event.sub_id),
        query: event.query,
    });
}

fn on_snapshot(
    event: On<Snapshot>,
    mut requests_query: BevyQuery<&mut SnapshotRequests>,
) {
    match requests_query.get_mut(event.view.inner()) {
        Ok(mut requests) => {
            requests.queue.push(SnapshotRequest {
                id: event.id,
                sub_id: None,
                query: event.query,
            });
        },

        Err(_) => {
            // Currently, our only consumer, the server plugin, always initializes this component.
            // So this todo will never be triggered.
            todo!();
        },
    }
}

fn on_unsubscribe(
    event: On<Unsubscribe>,
    mut subscriptions: BevyQuery<&mut Subscriptions>,
    mut commands: Commands,
) {
    let mut subscriptions = subscriptions.get_mut(event.view.inner()).unwrap();

    if subscriptions.queries.remove(&event.sub_id).is_none() {
        commands.trigger(SubscriptionRequestRejected {
            request: event.id,
            reason: SubscriptionRejectionReason::UnknownSubscription(event.sub_id),
        });
        return;
    }

    commands.trigger(SubscriptionRequestCompleted {
        request: event.id,
    });
}