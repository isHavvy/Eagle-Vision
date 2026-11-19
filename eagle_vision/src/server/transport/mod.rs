use crate::bevy::prelude::{App, Plugin};

pub mod transports;

pub trait TransportPlugin: Plugin {
    type StartupData: Send + 'static;

    fn install_startup_data(
        app: &mut App,
        data: Self::StartupData,
    );
}

pub struct Transport<T: TransportPlugin> {
    pub plugin: T,
    pub startup: T::StartupData,
}