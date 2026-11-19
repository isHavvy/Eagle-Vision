use crate::bevy::prelude::*;
use crate::engine_plugin::View;
use crate::subscription::SnapshotRequests;
use super::*;

#[derive(Debug, Default, Resource)]
pub struct PendingHellos(pub VecDeque<(SessionId, ClientHello)>);

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

#[derive(Debug, Resource)]
pub(super) struct SessionCounter(u64);

impl SessionCounter {
    pub fn next(&mut self) -> SessionId {
        let ret = SessionId(self.0);
        self.0 += 1;
        ret
    }
}

impl Default for SessionCounter {
    fn default() -> Self {
        Self(1)
    }
}

#[derive(Component)]
pub struct Session {
    pub id: SessionId,
    pub editor: EditorId,
    pub source: Option<SourceId>,
    pub readonly: bool,
}

#[derive(Debug, Default, Resource)]
pub struct HandshakeResults(VecDeque<(SessionId, Result<ServerWelcome, HandshakeError>)>);

pub fn handle_hello(world: &mut World, hello: ClientHello) -> Result<ServerWelcome, HandshakeError> {
    if hello.protocol_version != PROTOCOL_VERSION {
        Err(HandshakeError::VersionMismatch {
            client: hello.protocol_version, server: PROTOCOL_VERSION,
        })?;
    }

    // Locales: requested codes that exist, in request order, base appended.
    let mut all_locales = world.query::<(&Locale, &EagleVisionId)>();
    let mut returned_locales: Vec<(String, LocaleId)> = vec![];

    for requested_locale in &hello.locales {
        let found = all_locales.iter(world).find(|(c, _)| c.code().eq(requested_locale));

        if let Some((locale, id)) = found {
            if !returned_locales.iter().any(|(_, id2)| id2 == id) {
                // SAFETY: Aspect is a Locale because it's an ID on a component with a Locale component.
                returned_locales.push((locale.code().to_string(), unsafe { LocaleId::from_untyped_unchecked(*id) }));
            }
        }
    }

    let base_eid = BaseLocale::get(world);

    // SAFETY(unwrap): The BaseLocale component must be on a Locale entity, so it must have Locale and EagleVisionId components.
    let (base_locale, base_id) = all_locales.get(world, base_eid).unwrap();

    if !returned_locales.iter().any(|(_, id)| *id == *base_id) {
        // SAFETY: Aspect is a Locale because it's an ID on a component with a Locale component.
        returned_locales.push((base_locale.code().to_string(), unsafe { LocaleId::from_untyped_unchecked(*base_id) }));
    }

    // Editor: anonymous marker if no username, else find-or-create. Or at least, when implemented.
    let editor_eid = match &hello.username {
        None => EditorAnonymous::get(world),
        Some(_name) => {
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
            source: source.map(SourceId::from_nonzero),
            readonly: hello.readonly(),
        },

        View {
            locales: returned_locales.iter().map(|(_, i)| *i).collect(),
        },

        Outbox::default(),

        Subscriptions::default(),
        SnapshotRequests::default(),
    )).id();

    world.resource_mut::<Sessions>().insert(session_id, session_eid);

    Ok(ServerWelcome { session_id, editor_id, allocated_source: source, locales: returned_locales })
}

pub(super) fn accept_connections(world: &mut World) {
    let pending = std::mem::take(&mut world.resource_mut::<PendingHellos>().0);
    for (conn, hello) in pending {
        let result = handle_hello(world, hello);
        world.resource_mut::<HandshakeResults>().0.push_back((conn, result));
    }
}