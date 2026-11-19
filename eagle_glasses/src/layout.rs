use bevy::feathers::display::label;
use bevy::{ui_widgets::Activate};
use bevy::scene::SpawnSceneError;
use bevy::feathers::controls::{FeathersButton, FeathersTextInput, FeathersTextInputContainer};
use bevy::prelude::*;
use eagle_squak::protocol::{ClientHello, ClientHelloFlags};
use eagle_vision::eaglevision::EagleVision;

use bevy_form::prelude::*;
use bevy_form::field::readers::ReadEditableText;
use eagle_vision::init::EagleVisionInitialSettings;

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

fn new_project_form_scene() -> impl Scene {
    bsn! {
        #NewProjectForm
        on(on_new_project_submit)
        Node {
            align_self: AlignSelf::FlexStart,
            flex_direction: FlexDirection::Column,
        }

        Children [
            Node {
                flex_direction: FlexDirection::Row,
                margin: { px(4).all() },
            }

            Children [
                @label("Project Name")

                --

                @FeathersTextInputContainer {}
                Children [
                    @FeathersTextInput{
                        @visible_width: { Some(20.0) }
                    }
                    FieldOf { form: #NewProjectForm, name: "name" }
                    field_reader(ReadEditableText { discard_empty: true, ..default() })
                ]
            ]

            --

            Node {
                flex_direction: FlexDirection::Row,
                margin: { px(4).all() },
            }

            Children [
                @label("Locale")

                --

                @FeathersTextInputContainer {}
                
                Children [
                    @FeathersTextInput {
                        @visible_width: { Some(10.0) },
                    }
                    // Text::new("en-US")
                    FieldOf { form: #NewProjectForm, name: "locale" }
                    field_reader(ReadEditableText { discard_empty: true, ..default() })
                ]
            ]

            --

            Node {
                flex_direction: FlexDirection::Row,
                margin: { px(4).all() },
            }

            Children [
                @label("Your name")

                --

                @FeathersTextInputContainer {}

                Children [
                    @FeathersTextInput {
                        @visible_width: { Some(20.0) },
                    }
                    FieldOf { form: #NewProjectForm, name: "user" }
                    field_reader(ReadEditableText { discard_empty: true, ..default() })
                ]
            ]

            --

            @FeathersButton
            FormButton::Submit(#NewProjectForm)
            Text::new("Create Project")
        ]
    }
}

fn on_new_project_submit(
    form: On<Submit>,
    mut commands: Commands,
) {
    let mut init_settings = EagleVisionInitialSettings::new();

    if let Some(project_name) = form.value::<String>("name") {
        init_settings.set_project_name(project_name);
    }

    if let Some(locale) = form.value::<String>("locale") {
        init_settings.set_base_locale(locale);
    }

    if let Some(username) = form.value::<String>("user") {
        init_settings.set_creator(username);
    }

    let mut hello = ClientHello {
        username: form.value::<String>("user"),
        locales: form.value::<String>("locale").map(|code| vec![code]).unwrap_or(vec![]),
        ..Default::default()
    };
    hello.insert_flag(ClientHelloFlags::Readonly);

    commands.queue(|world: &mut World| {
        match EagleVision::new_project_with_single_client(init_settings, hello, false) {
            Ok((ev, _welcome)) => {
                world.insert_non_send(ev);
            },

            Err(err) => {
                warn!("Failed to create project: {:?}", err)
            },
        }

        world.commands().replace_main_content(bsn! { Node });
    });
}

#[derive(Debug)]
#[expect(unused)]
pub struct OpenProject(EagleVision);

impl std::ops::Deref for OpenProject {
    type Target = EagleVision;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for OpenProject {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}