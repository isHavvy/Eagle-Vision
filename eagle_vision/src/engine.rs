use std::assert_matches;
use std::thread::JoinHandle;

use bevy::app::{App, Plugin};

use crate::engine_plugin::{EagleVisionEnginePlugin, WorkPending};
use crate::init::{EagleVisionInitializationPlugin, NewProjectSettings};
use crate::schedule::{PostStartup, Startup, Update};
use crate::server::{EagleVisionServerPlugin, transport::{Transport, TransportPlugin}};

#[derive(Debug)]
pub struct EagleVision {
    app: App,
    state: EagleVisionState,
}

/// Typestate pattern for EagleVision, but only checked at runtime.
#[derive(Debug)]
enum EagleVisionState {
    Building {
        contains_transport_plugin: bool,
        contains_init: bool,
    },

    Started,

    Running,
}

impl EagleVision {
    pub fn new() -> Self {
        let mut app = App::empty();

        app.add_plugins((
            EagleVisionEnginePlugin,
            EagleVisionServerPlugin,
        ));

        Self {
            app,
            state: EagleVisionState::Building {
                contains_transport_plugin: false,
                contains_init: false,
            },
        }
    }

    pub fn set_new_project(&mut self, settings: NewProjectSettings) {
        self.state.assert_building_without_init();
        self.app.add_plugins(EagleVisionInitializationPlugin { settings });
        self.state.set_contains_initializer();
    }

    pub fn contains_init(&self) -> bool {
        self.state.assert_building();
        matches!(self.state, EagleVisionState::Building { contains_init: true, .. })
    }

    pub fn set_transport<T: TransportPlugin>(&mut self, transport: Transport<T>) -> &mut Self {
        self.state.assert_building_without_transport();

        self.app.add_plugins(transport.plugin);
        <T as TransportPlugin>::install_startup_data(&mut self.app, transport.startup);

        self.state.set_contains_transport_plugin();

        self
    }

    pub fn contains_transport(&self) -> bool {
        self.state.assert_building();
        matches!(self.state, EagleVisionState::Building { contains_transport_plugin: true, .. })
    }

    pub fn startup(&mut self) {
        self.state.assert_building();

        match self.state {
            EagleVisionState::Building {
                contains_transport_plugin: true,
                contains_init: true,
            } => { /* do nothing */},

            _ => {
                panic!("EagleVision startup requires project initialization and a transport");
            },
        }

        self.app.finish();
        self.app.cleanup();

        let world = self.app.world_mut();

        world.run_schedule(Startup);
        world.flush();
        world.clear_trackers();

        world.run_schedule(PostStartup);
        world.flush();
        world.clear_trackers();

        self.state = EagleVisionState::Started;
    }

    // #[TODO(Havvy)]: Better name.
    fn turn_internal(&mut self) {
        let world = self.app.world_mut();

        world.resource::<WorkPending>().clear();
        world.run_schedule(Update);
        world.flush();
        world.clear_trackers();
    }

    pub fn turn(&mut self) {
        assert!(matches!(self.state, EagleVisionState::Started | EagleVisionState::Running));
        self.state = EagleVisionState::Running;
        self.turn_internal();
    }

    pub fn run(mut self) -> ! {
        match self.state {
            EagleVisionState::Building { .. } => {
                self.startup();
            },
            EagleVisionState::Started => { /* do nothing */},
            EagleVisionState::Running => unreachable!(),
        }

        self.state = EagleVisionState::Running;

        // #[TODO(Havvy)]: We need a means by which the transport plugin can wake
        //                 the engine when it receives a message.
        //
        //                 Until then, we just sleep.
        //                 100ms chosen so that when that becomes painful enough,
        //                 I have incentive to actually do this.
        loop {
            self.turn_internal();

            if !self.app.world().resource::<WorkPending>().is_set() {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
}

impl EagleVisionState {
    fn assert_building(&self) {
        assert_matches!(self, EagleVisionState::Building { .. })
    }

    fn assert_started(&self) {
        assert_matches!(self, EagleVisionState::Started { .. })
    }

    fn assert_running(&self) {
        assert_matches!(self, EagleVisionState::Running { .. })
    }

    fn assert_building_without_init(&self) {
        assert_matches!(self, EagleVisionState::Building { contains_init: false, .. });
    }

    fn assert_building_without_transport(&self) {
        assert_matches!(self, EagleVisionState::Building { contains_transport_plugin: false, .. })
    }

    fn set_contains_initializer(&mut self) {
        match self {
            EagleVisionState::Building { contains_init: contains_initializer, .. } => {
                *contains_initializer = true;
            },
            EagleVisionState::Started => { /* do nothing */},
            EagleVisionState::Running => { /* do nothing */},
        }
    }

    fn set_contains_transport_plugin(&mut self) {
        match self {
            EagleVisionState::Building { contains_transport_plugin, .. } => {
                *contains_transport_plugin = true;
            },
            EagleVisionState::Started => { /* do nothing */},
            EagleVisionState::Running => { /* do nothing */},
        }
    }
}

// Using fully qualified names here because I'm not sure if this should ultimately live here.
#[derive(Debug)]
pub struct SingleStartupChannelsClientConnection {
    pub welcome: eagle_squak::protocol::ServerWelcome,
    pub channels: crate::server::transport::transports::single_startup_channels::SingleClientHandle,
    pub thread: JoinHandle<()>,
}

pub fn spawn_single_client_engine(
    settings: NewProjectSettings,
    hello: eagle_squak::protocol::ClientHello,
) -> Result<SingleStartupChannelsClientConnection, eagle_squak::protocol::HandshakeError>
{
    use crate::server::transport::transports::single_startup_channels::{SingleClientPlugin, SingleClientStartupData, SingleClientHandle};

    let (request_tx, request_rx) = std::sync::mpsc::channel();
    let (message_tx, message_rx) = std::sync::mpsc::channel();
    let (welcome_tx, welcome_rx) = std::sync::mpsc::channel();

    let thread = std::thread::Builder::new()
        .name("eagle-vision-engine".into())
        .spawn(move || {
            let mut ev = EagleVision::new();

            ev.set_new_project(settings);

            ev.set_transport(Transport {
                plugin: SingleClientPlugin,
                startup: SingleClientStartupData {
                    inbound: request_rx,
                    outbound: message_tx,
            }});

            ev.startup();

            let welcome = match crate::server::session::handle_hello(ev.app.world_mut(), hello) {
                Ok(welcome) => welcome,
                Err(err) => {
                    let _ = welcome_tx.send(Err(err));
                    return;
                }
            };

            let _ = welcome_tx.send(Ok(welcome));

            ev.run();
        })
        .expect("failed to spawn EagleVision engine thread");

    let welcome = welcome_rx
        .recv()
        .expect("engine thread died before handshake")?;

    Ok(SingleStartupChannelsClientConnection {
        welcome,
        channels: SingleClientHandle {
            requests: request_tx,
            messages: message_rx,
        },
        thread,
    })
}