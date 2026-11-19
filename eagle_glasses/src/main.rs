use bevy::feathers::dark_theme::create_dark_theme;
use bevy::feathers::theme::UiTheme;
use bevy::prelude::*;
use bevy::feathers::FeathersPlugins;

use bevy_form::FormPlugin;

mod layout;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(FeathersPlugins)
    .add_plugins(FormPlugin)
    .insert_resource(UiTheme(create_dark_theme()))
    .add_systems(Startup, spawn_layout_and_camera.spawn())
    .run();
}

fn spawn_layout_and_camera() -> impl SceneList {
    bsn_list! {
        Camera2d
        --
        @layout::layout_scene()
    }
}