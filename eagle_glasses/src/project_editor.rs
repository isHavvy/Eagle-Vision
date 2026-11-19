use bevy::feathers::controls::{
    FeathersButton, FeathersTextInput, FeathersTextInputContainer,
};
use bevy::feathers::display::label;
use bevy::prelude::*;
use bevy_form::prelude::*;
use bevy_form::field::readers::ReadEditableText;

use eagle_squak::protocol::{
    ClientRequest, ClientRequestBody,
};

use crate::{EagleVisionClient, OpenProject};

const PROJECT_NAME_FIELD: &str = "project name";

pub fn project_editor_scene() -> impl Scene {
    bsn! {
        #ProjectNameForm
        on(on_project_name_submit)

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
                    @FeathersTextInput {
                        @visible_width: { Some(30.0) }
                    }

                    FieldOf { form: #ProjectNameForm, name: PROJECT_NAME_FIELD }
                    field_reader(ReadEditableText { discard_empty: true, ..default() })
                ]
            ]

            --

            @FeathersButton
            FormButton::Submit(#ProjectNameForm)
            Text::new("Rename Project")
        ]
    }
}

fn on_project_name_submit(
    form: On<Submit>,
    mut client: ResMut<EagleVisionClient>,
    project: NonSend<OpenProject>,
) {
    let Some(name) = form.value::<String>(PROJECT_NAME_FIELD) else {
        return;
    };

    let request = ClientRequest {
        id: client.next_request_id(),
        body: ClientRequestBody::SetProjectName { name },
    };

    if let Err(err) = project.0.channels.requests.send(request) {
        warn!("Failed to send SetProjectName to EagleVision: {:?}", err);
    }
}