use std::collections::VecDeque;
use std::num::NonZeroU16;

use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy::ecs::schedule::ScheduleLabel;
use eagle_squak::id::{EagleVisionId, EditorId, LocaleId};
use eagle_squak::protocol::{ClientHello, ClientRequest, HandshakeError, PROTOCOL_VERSION, ServerMessage, ServerWelcome, SessionId};
use smallvec::SmallVec;

use crate::aspect::locale::Locale;
use crate::eaglevision::{GuaranteedMarkerComponent as _, SourceAllocator};
use crate::aspect::editor::{EditorAnonymous, EditorErased};
use crate::aspect::metadata::BaseLocale;
use crate::subscription::Subscriptions;

pub mod single;

/// The base server that provides the base abstractions that language speaks.
/// 
/// Transport servers (e.g. [`single::SingleClientPlugin`]) sit on top of this layer.
pub struct EagleVisionServerPlugin;

impl Plugin for EagleVisionServerPlugin {
    fn build(&self, app: &mut App) {
        app
        .init_schedule(EagleVisionUpdate)
        .init_resource::<IncomingRequests>()
        .init_resource::<SessionCounter>()
        .init_resource::<Sessions>()
        .configure_sets(EagleVisionUpdate, (
            LanguageSet::ReadRequests,
            LanguageSet::ExecuteRequests,
            LanguageSet::UpdateSubscriptions,
            LanguageSet::SendMessages,
        ).chain())
        ;
    }
}

/// EagleVision's version of Bevy's [`Update`]
#[derive(ScheduleLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EagleVisionUpdate;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageSet {
    HandleConnections,
    ReadRequests,
    ExecuteRequests,
    UpdateSubscriptions,
    SendMessages,
}

#[derive(Debug, Default, Resource)]
pub struct PendingHellos(pub VecDeque<(SessionId, ClientHello)>);

/// All requests that need to be processed. Filled by the transport server layer.
#[derive(Debug, Default, Resource)]
pub struct IncomingRequests(pub VecDeque<(SessionId, ClientRequest)>);

/// Per-session outbound, to be drained by the transport sever layer.
#[derive(Debug, Default, Component)]
pub struct OutgoingMessages(pub VecDeque<ServerMessage>);

#[derive(Debug, Default, Resource)]
pub struct Sessions {
    map: HashMap<SessionId, Entity>
}

impl Sessions {
    pub fn insert(&mut self, id: SessionId, entity: Entity) {
        self.map.insert(id, entity);
    }
}

impl std::ops::Index<SessionId> for Sessions {
    type Output = Entity;

    fn index(&self, index: SessionId) -> &Self::Output {
        &self.map[&index]
    }
}

#[derive(Debug, Default, Resource)]
struct SessionCounter(u64);

impl SessionCounter {
    fn next(&mut self) -> SessionId {
        let ret = SessionId(self.0);
        self.0 += 1;
        ret
    }
}

#[derive(Component)]
pub struct Session {
    pub id: SessionId,
    pub editor: EditorId,
    pub source: Option<NonZeroU16>,
    pub locales: SmallVec<[LocaleId; 2]>,
    pub readonly: bool,
}

#[derive(Debug, Default, Resource)]
pub struct HandshakeResults(VecDeque<(SessionId, Result<ServerWelcome, HandshakeError>)>);

pub fn handle_hello(world: &mut World, hello: ClientHello) -> Result<ServerWelcome, HandshakeError> {
    if hello.protocol_version != PROTOCOL_VERSION {
        return Err(HandshakeError::VersionMismatch {
            client: hello.protocol_version, server: PROTOCOL_VERSION,
        });
    }

    // Locales: requested codes that exist, in request order, base appended.
    let mut known_locales = world.query::<(&Locale, &EagleVisionId)>();
    let mut locales: Vec<(String, LocaleId)> = Vec::new();
    for want in &hello.locales {
        let found = known_locales.iter(world).find(|(c, _)| c.code().eq(want));
        if let Some((locale, id)) = found {
            if !locales.iter().any(|(_, id2)| id2 == id) {
                // SAFETY: Aspect is a Locale because it's an ID on a component with a Locale component.
                locales.push((locale.code().to_string(), unsafe { LocaleId::from_untyped_unchecked(*id) }));
            }
        }
    }
    let base_eid = BaseLocale::get(world);
    // SAFETY(unwrap): The BaseLocale component must be on a Locale entity, so it must have Locale and EagleVisionId components.
    let (base_locale, base_id) = known_locales.get(world, base_eid).unwrap();
    if !locales.iter().any(|(_, id)| *id == *base_id) {
        // SAFETY: Aspect is a Locale because it's an ID on a component with a Locale component.
        locales.push((base_locale.code().to_string(), unsafe { LocaleId::from_untyped_unchecked(*base_id) }));
    }

    // Editor: anonymous marker if no username, else find-or-create. Or at least, when implemented.
    let editor_eid = match &hello.username {
        None => EditorAnonymous::get(world),
        Some(name) => {
            // #[TODO(Havvy): find_or_create_editor(world, name, &locales),
            EditorErased::get(world)
        }
    };
    let editor_id = *world.query::<&EagleVisionId>().get(world, editor_eid).unwrap();
    // SAFETY: We know that this id belongs to an Editor by this point.
    let editor_id = unsafe { EditorId::from_untyped_unchecked(editor_id) };

    let source = if hello.readonly() {
        None
    } else {
        match world.resource_mut::<SourceAllocator>().next() {
            Some(source) => Some(source),
            None => Err(HandshakeError::SourcesExhausted)?,
        }
    };
    let session_id = world.resource_mut::<SessionCounter>().next();

    let session_eid = world.spawn((
        Session {
            id: session_id,
            editor: editor_id,
            source,
            readonly: hello.readonly(),
            locales: locales.iter().map(|(_, i)| *i).collect()
        },
        OutgoingMessages::default(),
        Subscriptions::default(),
    )).id();
    world.resource_mut::<Sessions>().insert(session_id, session_eid);

    Ok(ServerWelcome { session_id, editor_id, allocated_source: source, locales })
}

fn accept_connections(world: &mut World) {
    let pending = std::mem::take(&mut world.resource_mut::<PendingHellos>().0);
    for (conn, hello) in pending {
        let result = handle_hello(world, hello);
        world.resource_mut::<HandshakeResults>().0.push_back((conn, result));
    }
}