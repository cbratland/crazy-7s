use super::{MenuState, ServerState};
use bevy::prelude::*;
use rand::Rng;

const TEXT_COLOR: Color = Color::WHITE;

/// Indicates that the component bundle is for this screen.
#[derive(Component)]
pub struct OnScreen;

/// Indicates the bundle's associated button action.
#[derive(Component)]
pub enum ButtonAction {
    Host,
    Join,
    Settings,
}

/// Draws the main menu.
pub fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Common style for all buttons on the screen
    let button_style = Style {
        width: Val::Px(274.0),
        height: Val::Px(72.0),
        margin: UiRect::all(Val::Px(10.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            },
            OnScreen,
        ))
        .with_children(|parent| {
            parent.spawn((
                ButtonBundle {
                    style: Style {
                        position_type: PositionType::Absolute,
                        top: Val::Px(26.0),
                        right: Val::Px(26.0),
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        ..default()
                    },
                    background_color: Color::WHITE.into(),
                    image: UiImage::new(asset_server.load("textures/buttons/settings.png")),
                    ..default()
                },
                ButtonAction::Settings,
            ));

            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Display the game name
                    parent.spawn(
                        TextBundle::from_section(
                            "crazy 7s",
                            TextStyle {
                                font: asset_server.load("fonts/Lato-BlackItalic.ttf"),
                                font_size: 122.0,
                                color: TEXT_COLOR,
                                ..default()
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(30.0)),
                            ..default()
                        }),
                    );

                    // show buttons
                    parent.spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: Color::WHITE.into(),
                            image: asset_server.load("textures/buttons/host.png").into(),
                            ..default()
                        },
                        ButtonAction::Host,
                    ));

                    parent.spawn((
                        ButtonBundle {
                            style: button_style.clone(),
                            background_color: Color::WHITE.into(),
                            image: asset_server.load("textures/buttons/join.png").into(),
                            ..default()
                        },
                        ButtonAction::Join,
                    ));
                });
        });
}

/// Handles button presses.
pub fn handle_action(
    interaction_query: Query<&ButtonAction, (Changed<Interaction>, With<Button>)>,
    mouse: Res<Input<MouseButton>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut server_state: ResMut<NextState<ServerState>>,
) {
    for menu_button_action in &interaction_query {
        if mouse.just_released(MouseButton::Left) {
            match menu_button_action {
                ButtonAction::Host => {
                    let code = rand::thread_rng().gen_range(1000..10000);
                    server_state.set(ServerState::Server(code));
                    menu_state.set(MenuState::Lobby);
                }
                ButtonAction::Join => {
                    menu_state.set(MenuState::Join);
                }
                ButtonAction::Settings => {
                    menu_state.set(MenuState::Settings);
                }
            }
        }
    }
}
