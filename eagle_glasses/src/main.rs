use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::theme::UiTheme;
use bevy::prelude::*;
use bevy::feathers::FeathersPlugins;

use bevy_form::FormPlugin;
use eagle_squak::protocol::{RequestId, ServerMessage, ServerWelcome, Snapshot, SubscriptionId, subscription::Diff};
use eagle_vision::engine::SingleStartupChannelsClientConnection as EvConnection;

use crate::subscriptions::ProjectNameCache;

pub mod layout;
pub mod new_project_form;
pub mod project_editor;
pub mod subscriptions;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "My Game Title".into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(FeathersPlugins)
    .add_plugins(FormPlugin)
    .insert_resource(UiTheme(create_dark_theme()))
    .init_resource::<ProjectNameCache>()
    .add_systems(Startup, spawn_layout_and_camera.spawn())
    .add_systems(Update, drain_eagle_vision_messages)
    .add_systems(Update, update_window_title)
    .add_observer(new_project_form::on_ev_started)
    .run();
}

fn spawn_layout_and_camera() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @layout::layout_scene()
    }
}

#[derive(Debug)]
pub struct OpenProject(EvConnection);

impl std::ops::Deref for OpenProject {
    type Target = EvConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for OpenProject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Resource)]
pub struct EagleVisionClient {
    pub welcome: ServerWelcome,
    next_request: u64,
    next_subscription: u32,
    pub project_name_sub: Option<SubscriptionId>,
}

impl EagleVisionClient {
    pub fn new(welcome: ServerWelcome) -> Self {
        Self {
            welcome,
            next_request: 1,
            next_subscription: 1,
            project_name_sub: None,
        }
    }

    pub fn next_request_id(&mut self) -> RequestId {
        let id = RequestId(self.next_request);
        self.next_request += 1;
        id
    }

    pub fn next_subscription_id(&mut self) -> SubscriptionId {
        let id = SubscriptionId(self.next_subscription);
        self.next_subscription += 1;
        id
    }
}

fn drain_eagle_vision_messages(
    project: Option<NonSend<OpenProject>>,
    mut project_name: ResMut<ProjectNameCache>,
    client: Option<Res<EagleVisionClient>>,
) {
    let Some(project) = project else {
        return;
    };

    while let Ok(message) = project.0.channels.messages.try_recv() {
        match message {
            ServerMessage::Snapshot { id: _id, snapshot: Snapshot {sub_id, items }} => {
                if sub_id == client.as_ref().and_then(|c| c.project_name_sub) {
                    for item in items {
                        project_name.apply_item(item);
                    }
                }
            }

            ServerMessage::SubscriptionDiff { sub_id, diff: Diff { added, updated, removed } } => {
                if Some(sub_id) == client.as_ref().and_then(|c| c.project_name_sub) {
                    for item in added {
                        project_name.apply_item(item);
                    }

                    for item in updated {
                        project_name.apply_item(item);
                    }

                    for id in removed {
                        project_name.remove(id);
                    }
                }
            }

            ServerMessage::CommandComplete { id: revision } => {
                info!("EagleVision request complete: {:?}", revision);
            }

            ServerMessage::RequestRejected { rejection } => {
                warn!("EagleVision request rejected: {:?}", rejection);
            }
        }
    }
}

fn update_window_title(
    project_name: Res<ProjectNameCache>,
    mut windows: Query<&mut Window>,
) {
    if !project_name.is_changed() {
        return;
    }

    let Ok(mut window) = windows.single_mut() else {
        return;
    };

    window.title = match project_name.value.as_deref() {
        Some(name) => format!("Eagle Glasses - {name}"),
        None => "Eagle Glasses".to_string(),
    };
}