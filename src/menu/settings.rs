use super::MenuState;
use crate::{storage::Storage, Username};
use bevy::prelude::*;

/// Username text component.
#[derive(Component)]
pub struct UsernameText;

/// Indicates that the component bundle is for this screen.
#[derive(Component)]
pub struct OnScreen;

/// Indicates the bundle's associated button action.
#[derive(Component, Clone, Copy)]
pub enum ButtonAction {
    BackToMain,
}

/// Draws settings screen.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
                                value: String::from("Username:\n"),
                                style: text_style.clone(),
                            },
                            TextSection {
                                value: String::new(),
                                style: text_style.clone(),
                            },
                        ],
                        ..default()
                    },
                    ..default()
                },
                UsernameText,
            ));
        });
}

/// Updates stored username on key press.
pub fn update_name(
    mut char_evr: EventReader<ReceivedCharacter>,
    mut name: ResMut<Username>,
    keys: Res<Input<KeyCode>>,
) {
    let name = &mut name.0;
    if keys.just_pressed(KeyCode::Back) {
        name.pop();
    } else {
        for ev in char_evr.read() {
            if name.len() < 15 && (ev.char.is_alphanumeric() || ev.char == '_' || ev.char == ' ') {
                name.push(ev.char);
            }
        }
    }
}

/// Copies stored username to text display.
pub fn update_name_display(mut text: Query<&mut Text, With<UsernameText>>, name: Res<Username>) {
    let mut text = text.single_mut();
    text.sections[1].value = name.0.clone();
}

/// Handles button presses.
pub fn handle_action(
    interaction_query: Query<&ButtonAction, (Changed<Interaction>, With<Button>)>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut storage: ResMut<Storage>,
    mouse: Res<Input<MouseButton>>,
    name: Res<Username>,
) {
    for action in interaction_query.iter() {
        if mouse.just_released(MouseButton::Left) {
            match action {
                ButtonAction::BackToMain => {
                    menu_state.set(MenuState::Main);
                    storage
                        .set("username", &name.0)
                        .expect("failed to save username");
                }
            }
        }
    }
}
