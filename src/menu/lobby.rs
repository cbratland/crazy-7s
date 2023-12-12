use super::{MenuState, ServerState};
use crate::info::{Opponent, Opponents};
use crate::network::{PeerNames, StartGame};
use crate::SERVER_URL;
use bevy::prelude::*;
use bevy_matchbox::prelude::*;

/// Player count text component.
#[derive(Component)]
pub struct PlayersText;

/// Indicates that the component bundle is for this screen.
#[derive(Component)]
pub struct OnScreen;

/// Indicates the bundle's associated button action.
#[derive(Component, Clone, Copy)]
pub enum ButtonAction {
    Back,
    Start,
}

/// Draws lobby screen and connects to the server.
pub fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    server_state: Res<State<ServerState>>,
) {
    let text_style = TextStyle {
        font: asset_server.load("fonts/Lato-Black.ttf"),
        font_size: 40.0,
        color: Color::WHITE,
    };

    let server_state = *server_state.get();
    let code = match server_state {
        ServerState::Server(code) => code,
        ServerState::Client(code) => code,
        _ => 0,
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
                ButtonAction::Back,
            ));

            // room code text
            parent.spawn(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                text: Text::from_section(format!("Room {code}"), text_style.clone()),
                ..Default::default()
            });

            // players text
            parent.spawn((
                TextBundle {
                    style: Style {
                        align_self: AlignSelf::Center,
                        justify_content: JustifyContent::Center,
                        ..Default::default()
                    },
                    text: Text::from_section("Players: 1", text_style),
                    ..Default::default()
                },
                PlayersText,
            ));

            // start button
            if let ServerState::Server(_) = server_state {
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
                        image: asset_server.load("textures/buttons/start.png").into(),
                        ..default()
                    },
                    ButtonAction::Start,
                ));
            }
        });

    start_socket(commands, code);
}

/// Connects to the server.
fn start_socket(mut commands: Commands, code: u16) {
    let room_url = format!("{SERVER_URL}/v1_{code}");
    commands.insert_resource(MatchboxSocket::new_reliable(room_url));
}

/// Closes the server connection.
pub fn close_socket(mut commands: Commands) {
    commands.remove_resource::<MatchboxSocket<SingleChannel>>();
}

/// Updates the player count text.
pub fn update_players_text(
    mut query: Query<&mut Text, With<PlayersText>>,
    socket: Res<MatchboxSocket<SingleChannel>>,
) {
    let count = socket.connected_peers().collect::<Vec<_>>().len() + 1;
    let mut text = query.single_mut();
    text.sections[0].value = format!("Players: {count}");
}

/// Handles button presses.
pub fn handle_action(
    interaction_query: Query<&ButtonAction, (Changed<Interaction>, With<Button>)>,
    mut start_events: EventWriter<StartGame>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut server_state: ResMut<NextState<ServerState>>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut opponents: ResMut<Opponents>,
    mouse: Res<Input<MouseButton>>,
    peer_names: Res<PeerNames>,
) {
    for menu_button_action in &interaction_query {
        if mouse.just_released(MouseButton::Left) {
            match menu_button_action {
                ButtonAction::Back => {
                    menu_state.set(MenuState::Main);
                    server_state.set(ServerState::None);
                }
                ButtonAction::Start => {
                    // get peer ids and randomly shuffle for player order
                    let Some(own_pid) = socket.id() else { return; };
                    let mut order = socket.connected_peers().collect::<Vec<_>>();

                    order.push(own_pid);
                    use rand::seq::SliceRandom;
                    use rand::thread_rng;
                    order.shuffle(&mut thread_rng());

                    // set opponents
                    opponents.0 = order
                        .iter()
                        .filter_map(|pid| {
                            if *pid == own_pid {
                                None
                            } else {
                                Some(Opponent::new(
                                    *pid,
                                    peer_names
                                        .0
                                        .get(pid)
                                        .cloned()
                                        .unwrap_or_else(|| String::from("Unknown")),
                                    5,
                                ))
                            }
                        })
                        .collect();

                    // send start game event to connected peers
                    start_events.send(StartGame {
                        order,
                        restart: false,
                    });
                }
            }
        }
    }
}
