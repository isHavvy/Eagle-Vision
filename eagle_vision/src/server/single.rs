//! Single client server transport over mpsc channels.

use std::sync::mpsc::Receiver;

use bevy::prelude::*;
use eagle_squak::protocol::{ClientRequest, SessionId};

use crate::server::{IncomingRequests, OutgoingMessages};

use super::{EagleVisionUpdate, LanguageSet};

pub struct SingleClientPlugin;

impl Plugin for SingleClientPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(EagleVisionUpdate, (
                read_requests.in_set(LanguageSet::ReadRequests),
            ))
        ;
    }
}

impl SingleClientPlugin {
    const SESSION_ID: SessionId = SessionId(1);
}

pub struct ClientRequestReceiver(Receiver<ClientRequest>);

fn read_requests(
    receiver: NonSendMut<ClientRequestReceiver>,
    mut incoming_requests: ResMut<IncomingRequests>,
) {
    while let Ok(request) = receiver.0.try_recv() {
        incoming_requests.0.push_back((SingleClientPlugin::SESSION_ID, request));
    }
}