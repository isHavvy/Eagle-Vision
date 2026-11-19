use std::collections::HashSet;
use std::num::NonZeroU16;

use crate::aspect::{Editor, Locale};
use crate::id::{AspectedId, EagleVisionId};
use crate::subscription::{SubscriptionTarget, SubscriptionId};

pub const PROTOCOL_VERSION: u32 = 0;

/// The client's session id. Useful for servers that allow multiple clients.
/// Currently, the server doesn't. But it will.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash,)]
pub struct SessionId(pub u64);

/// An id attached to requests to track which request it is.
/// 
/// It is up to the client to send unique ids.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash,)]
pub struct RequestId(pub u64);

#[derive(Debug, Clone)]
pub struct ClientHello {
    pub protocol_version: u32,

    pub flags: HashSet<ClientHelloFlags>,

    pub username: Option<String>,

    /// Ordered preferred locale codes (e.g., ["en-US", "en"]).
    /// 
    /// If none of your preffered locales are available, the Welcome will give
    /// your client the base locale of the project.
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
    pub editor_id: AspectedId<Editor>,

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
    pub locales: Vec<(String, AspectedId<Locale>)>,

    // #[TODO(Havvy)]: pub starting_subscriptions: Vec<(SubscriptionId, SubscriptionTarget)>,
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

#[derive(Debug, Clone)]
pub enum ClientRequestBody {
    Subscribe {
        id: SubscriptionId,
        spec: SubscriptionTarget,
    },

    Unsubscribe {
        id: SubscriptionId,
    },

    SetProjectName {
        new_name: String,
    },
}

#[derive(Debug, Clone)]
pub enum ServerMessage {
    /// Request cannot be fulfilled.
    RequestRejected {
        rejection: RequestRejection,
    },

    /// Sent once all changes a request causes are done.
    CommandComplete {
        revision: RequestId,
    },

    /// What a subscription looks like when it is first subscribed to.
    SubscriptionSnapshot {
        sub_id: SubscriptionId,
        items: Vec<()>,
    },

    /// Changes to subscribed data.
    SubscriptionUpdate {
        sub_id: SubscriptionId,
        added: Vec<()>,
        updated: Vec<()>,
        removed: Vec<EagleVisionId>,
    },

    UpdateText {
        target: EagleVisionId,
        value: String,
    }
}

#[derive(Debug, Clone)]
pub struct RequestRejection {
    pub id: RequestId,
    pub reason: RejectionReason
}

#[derive(Debug, Clone)]
pub enum RejectionReason {
    EditCommandInReadonlyMode
}