use super::ButtonEnabled;
use super::MenuState;
use super::ServerState;
use bevy::prelude::*;

/// The code entered by the user.
#[derive(Resource)]
pub struct Code(String);

/// The text that displays the code.
#[derive(Component)]
pub struct CodeText;

/// Indicates that the component bundle is for this screen.
#[derive(Component)]
pub struct OnScreen;

/// Indicates the bundle's associated button action.
#[derive(Component, Clone, Copy)]
pub enum ButtonAction {
    BackToMain,
    Join,
}

/// Draws the join screen and initializes code resource.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(Code(String::default()));

    let text_style = TextStyle {
        font: asset_server.load("fonts/Lato-Black.ttf"),
        font_size: 40.0,
        color: Color::WHITE,
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnScreen,
        ))
        .with_children(|parent| {
            // back button
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(26.0),
                        left: Val::Px(26.0),
                        width: Val::Px(120.0),
                        height: Val::Px(46.0),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    image: asset_server.load("textures/buttons/back.png").into(),
                    ..default()
                },
                ButtonAction::BackToMain,
            ));

            // enter id text
            parent.spawn((
                TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    text: Text {
                        sections: vec![
                            TextSection {
                                value: String::from("Enter Room ID:\n"),
                                style: text_style.clone(),
                            },
                            TextSection {
                                value: String::from(" _ _ _ _"),
                                style: text_style.clone(),
                            },
                        ],
                        alignment: TextAlignment::Center,
                        ..default()
                    },
                    ..default()
                },
                CodeText,
            ));

            // start button
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        width: Val::Px(274.0),
                        height: Val::Px(72.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    image: asset_server.load("textures/buttons/join.png").into(),
                    ..default()
                },
                ButtonAction::Join,
                ButtonEnabled(false),
            ));
        });
}

/// Updates stored code on key press.
pub fn update_code(
    mut char_evr: EventReader<ReceivedCharacter>,
    mut code: ResMut<Code>,
    keys: Res<Input<KeyCode>>,
) {
    let code = &mut code.0;
    for ev in char_evr.read() {
        if code.len() < 4 && ev.char.is_ascii_digit() {
            code.push(ev.char);
        }
    }
    if keys.just_pressed(KeyCode::Back) {
        let mut chars = code.chars();
        chars.next_back();
        *code = chars.as_str().to_owned();
    }
}

/// Updates the displayed code text.
pub fn update_code_display(mut text: Query<&mut Text, With<CodeText>>, code: ResMut<Code>) {
    let mut text = text.single_mut();
    // fills unused digits with underscores
    let mut code = code.0.clone();
    for _ in 0..(4 - code.len()) {
        code.push(' ');
        code.push('_');
    }
    text.sections[1].value = code;
}

/// Enables or disables the start button depending on if code is 4 digits long or not.
pub fn update_button_enabled(mut buttons: Query<&mut ButtonEnabled>, code: ResMut<Code>) {
    let mut button = buttons.single_mut();
    button.0 = code.0.len() == 4;
}

/// Handles button presses.
pub fn handle_action(
    interaction_query: Query<
        (&ButtonAction, Option<&ButtonEnabled>),
        (Changed<Interaction>, With<Button>),
    >,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut server_state: ResMut<NextState<ServerState>>,
    mouse: Res<Input<MouseButton>>,
    code: Res<Code>,
) {
    for (action, enabled) in &interaction_query {
        if enabled.map_or(true, |e| e.0) && mouse.just_released(MouseButton::Left) {
            match action {
                ButtonAction::BackToMain => {
                    menu_state.set(MenuState::Main);
                }
                ButtonAction::Join => {
                    let code = code.0.parse::<u16>().expect("integer");
                    server_state.set(ServerState::Client(code));
                    menu_state.set(MenuState::Lobby);
                }
            }
        }
    }
}
