use std::collections::HashSet;
use std::num::NonZeroU16;

use crate::id::{EagleVisionId, Existing, New, SourceId};
use crate::aspect_ids::*;
use crate::protocol::subscription::{Diff, SubscriptionItem};
use crate::query::Query;

pub mod subscription;

pub const PROTOCOL_VERSION: u32 = 0;

/// The client's session id.
// #[TODO(Havvy)]: Is this a squak responsibility? Do clients need to know their session id?
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash,)]
pub struct SessionId(pub u64);

/// An id attached to requests to track which request it is.
/// 
/// It is up to the client to send unique ids.
// #[TODO(Havvy)]: Figure out what happens in the server plugin if this is false.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash,)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone)]
pub struct ClientHello {
    pub protocol_version: u32,

    pub flags: HashSet<ClientHelloFlags>,

    pub username: Option<String>,

    /// Ordered preferred locale codes (e.g., ["en-US", "en"])
    pub locales: Vec<String>,
}

impl Default for ClientHello {
    fn default() -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            flags: Default::default(),
            username: Default::default(),
            locales: Default::default()
        }
    }
}

impl ClientHello {
    pub fn insert_flag(&mut self, flag: ClientHelloFlags) -> &mut Self {
        self.flags.insert(flag);
        self
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    pub fn contains_flag(&self, flag: ClientHelloFlags) -> bool {
        self.flags.contains(&flag)
    }

    pub fn readonly(&self) -> bool {
        self.contains_flag(ClientHelloFlags::Readonly)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClientHelloFlags {
    /// When sent, the client cannot send editing commands.
    /// 
    /// The Welcome will not give a source ID.
    Readonly
}

#[derive(Debug, Clone,)]
pub struct ServerWelcome {
    pub session_id: SessionId,

    /// The ID corresponding to who you are.
    /// If no username was given, you'll be given the aspect id for the anonymous editor.
    pub editor_id: EditorId,

    /// Ephemeral source id assigned exclusively to this client connection for minting IDs.
    /// 
    /// In readonly mode, and only in readonly mode, this is None.
    pub allocated_source: Option<NonZeroU16>,

    /// Mapping of requested locale codes in the project to their `AspectedId<Locale>`, if
    /// they exist. Also includes the base locale at the end.
    /// 
    /// When receiving localized text, your client will receive the value of the first locale
    /// that matches this list. If a localized text does not have a value from any of these
    /// locales, an arbitrary value will given instead.
    pub locales: Vec<(String, LocaleId)>,

    // #[TODO(Havvy)]: pub starting_subscriptions: Vec<(SubscriptionId, Query)>,
}

#[derive(Debug, Clone,)]
pub enum HandshakeError {
    VersionMismatch { client: u32, server: u32 },
    SourcesExhausted,
}

#[derive(Debug, Clone)]
pub struct ClientRequest {
    pub id: RequestId,
    pub body: ClientRequestBody,
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum ClientRequestBody {
    Snapshot {
        query: Query,
    },

    /// This will also send a snapshot of the query before
    /// giving the CommandComplete
    Subscribe {
        id: SubscriptionId,
        query: Query,
    },

    Unsubscribe {
        id: SubscriptionId,
    },

    SetProjectName {
        name: String,
    },

    CreateAttributeDefinition {
        id: New<AttributeDefinitionId>
    },

    CreateAttribute {
        id: New<AttributeId>,
        definition: Existing<AttributeDefinitionId>,
        on: Existing<EagleVisionId>
    },

    CreateComment { comment: New<CommentId>, body: Existing<TextId>, on: Existing<EagleVisionId> },

    CreateText {
        text: String,
    }
}

impl ClientRequestBody {
    pub fn is_readonly(&self) -> bool {
        match *self {
            ClientRequestBody::Snapshot { .. } => true,
            ClientRequestBody::Subscribe { .. } => true,
            ClientRequestBody::Unsubscribe { .. } => true,
            ClientRequestBody::SetProjectName { .. } => false,
            ClientRequestBody::CreateAttributeDefinition { .. } => false,
            ClientRequestBody::CreateComment { .. } => false,
            ClientRequestBody::CreateAttribute { .. } => false,
            ClientRequestBody::CreateText { .. } => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum ServerMessage {
    /// Request cannot be fulfilled.
    RequestRejected {
        rejection: RequestRejection,
    },

    /// Sent once all changes a request causes are done.
    CommandComplete {
        id: RequestId,
    },

    /// Query result. Sent either by a Snapshot request or Subscription
    /// request before any subscription diffs are sent.
    Snapshot {
        id: RequestId,
        snapshot: Snapshot,
    },

    /// Changes to subscribed data.
    SubscriptionDiff {
        sub_id: SubscriptionId,
        diff: Diff,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct RequestRejection {
    pub id: RequestId,
    pub reason: RejectionReason
}

#[derive(Debug, Clone, Copy)]
pub enum RejectionReason {
    CommandUnimplemented,
    WriteCommandInReadonlyMode,
    IdAlreadyExists { id: EagleVisionId },
    IdNotFound { id: EagleVisionId },
    InvalidId(InvalidIdReason)
}

#[derive(Debug, Clone, Copy)]
pub enum InvalidIdReason {
    SourceNotOwned {
        attempted_source: SourceId,
        owned_source: SourceId,
    }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    /// If a snapshot is sent as part of subscribing to a query,
    /// this field will contain the subscription id that was sent
    /// with the subscribe request.
    pub sub_id: Option<SubscriptionId>,

    pub items: Vec<SubscriptionItem>,
}

/// An id for subscribing to information
/// 
/// The client provides which ID to use for each subscription with the subscribe command.
/// 
/// When 
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash,)]
pub struct SubscriptionId(pub u32);