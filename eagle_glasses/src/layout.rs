use bevy::ui_widgets::Activate;
use bevy::scene::SpawnSceneError;
use bevy::feathers::controls::FeathersButton;
use bevy::prelude::*;

use crate::new_project_form::new_project_form_scene;

#[derive(Debug, Clone, Copy, Default, Component)]
struct MainContent;

struct ReplaceMainContentCommand {
    contents: Box<dyn Scene>,
}

impl Command for ReplaceMainContentCommand {
    type Out = Result<(), SpawnSceneError>;

    fn apply(self, world: &mut World) -> Self::Out {
        let main_content = world
            .query_filtered::<Entity, With<MainContent>>()
            .single(world)
            .expect("Replace main content command called when there is no main content.");

        let new_contents = world.spawn_scene(self.contents)?.id();
        world.commands().entity(main_content).despawn_children().add_child(new_contents);

        Ok(())
    }
}

pub trait ReplaceMainContentExt {
    fn replace_main_content(&mut self, contents: impl Scene);
}

impl ReplaceMainContentExt for Commands<'_, '_> {
    fn replace_main_content(&mut self, contents: impl Scene) {
        self.queue(ReplaceMainContentCommand {
            contents: Box::new(contents) as Box<dyn Scene>
        });
    }
}

pub fn layout_scene() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
        }

        Children [
            #MainMenu
            Node {
                height: px(20),
                flex_direction: FlexDirection::Row,
            }

            Children [
                Text::new("Main Menu Here")
            ]

            --

            #MainContent
            MainContent
            Node {
                width: percent(100),
                flex_grow: 1.0,
            }
            Children [
                @no_open_project_scene()
            ]

            --

            #StatusBar
            Node {
                height: px(20),
            }
        ]
    }
}

fn no_open_project_scene() -> impl Scene {
    bsn! {
        Node {
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
        }

        Children [
            Node {
                margin: UiRect::new(px(0), px(0), px(0), px(20))
                align_self: AlignSelf::Center,
            }
            Text::new("No open project? Create a new project?")

            --

            Node {
                align_self: AlignSelf::Center,
            }
            @FeathersButton {}
            Text::new("New project")
            on(on_new_project_activate)
        ]
    }
}

fn on_new_project_activate(
    _event: On<Activate>,
    mut commands: Commands,
) {
    commands.replace_main_content(new_project_form_scene());
}