use bevy::feathers::controls::{FeathersButton, FeathersTextInput, FeathersTextInputContainer};
use bevy::prelude::*;
use bevy::feathers::display::label;

use bevy_form::prelude::*;
use bevy_form::field::readers::ReadEditableText;
use eagle_squak::protocol::{ClientHello, ClientRequest, ClientRequestBody};
use eagle_squak::query::Query;
use eagle_vision::engine::spawn_single_client_engine;
use eagle_vision::init::NewProjectSettings;

use crate::{EagleVisionClient, OpenProject};
use crate::layout::ReplaceMainContentExt as _;

const PROJECT_NAME_FIELD: &str = "project name field";
const LOCALE_FIELD: &str = "locale field";
const USERNAME_FIELD: &str = "username field";

pub fn new_project_form_scene() -> impl Scene {
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
                    FieldOf { form: #NewProjectForm, name: PROJECT_NAME_FIELD }
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
                    FieldOf { form: #NewProjectForm, name: LOCALE_FIELD }
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
                    FieldOf { form: #NewProjectForm, name: USERNAME_FIELD }
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
    let mut init_settings = NewProjectSettings::new();

    if let Some(project_name) = form.value::<String>(PROJECT_NAME_FIELD) {
        init_settings.set_project_name(project_name);
    }

    if let Some(locale) = form.value::<String>(LOCALE_FIELD) {
        init_settings.set_base_locale(locale);
    }

    if let Some(username) = form.value::<String>(USERNAME_FIELD) {
        init_settings.set_creator(username);
    }

    let hello = ClientHello {
        username: form.value::<String>(USERNAME_FIELD),
        locales: form.value::<String>(LOCALE_FIELD).map(|code| vec![code]).unwrap_or(vec![]),
        ..Default::default()
    };

    commands.queue(|world: &mut World| {
        match spawn_single_client_engine(init_settings, hello) {
            Ok(connection) => {
                world.insert_resource(EagleVisionClient::new(connection.welcome.clone()));
                world.insert_non_send(OpenProject(connection));
                world.trigger(EvStarted);

                world.commands().replace_main_content(crate::project_editor::project_editor_scene());
            },

            Err(err) => {
                warn!("Failed to create project: {:?}", err);
            },
        }
    });
}

#[derive(Debug, Event)]
pub struct EvStarted;

pub fn on_ev_started(
    _event: On<EvStarted>,
    mut client: ResMut<EagleVisionClient>,
    connection: NonSendMut<OpenProject>,
) {
    let sub_id = client.next_subscription_id();
    client.project_name_sub = Some(sub_id);

    let req = ClientRequest {
        id: client.next_request_id(),
        body: ClientRequestBody::Subscribe {
            id: sub_id,
            query: Query::ProjectName,
        }
    };

    if let Err(err) = connection.channels.requests.send(req) {
        warn!("Failed to subscribe to project name: {:?}", err);
    }
}