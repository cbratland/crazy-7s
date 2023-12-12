//! Win/lose screen.

use crate::{
    despawn_screen,
    menu::MenuState,
    network::{RestartGame, ServerState},
    GameScreenState, ScreenState,
};
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_matchbox::prelude::*;

/// Win event posted locally when a player wins.
#[derive(Event)]
pub struct Win(pub PeerId);

/// Indicates that the component bundle is for this screen.
#[derive(Component)]
pub struct OnScreen;

/// Indicates the bundle's associated button action.
#[derive(Component)]
pub enum ButtonAction {
    PlayAgain,
    Quit,
}

/// Draws win screen when Win event is received.
fn handle_win(
    mut events: EventReader<Win>,
    mut game_screen_state: ResMut<NextState<GameScreenState>>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    server_state: Res<State<ServerState>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let Some(Win(id)) = events.read().next() else { return; };
    let is_self = socket.id() == Some(*id);
    game_screen_state.set(GameScreenState::Win);

    // draw win screen
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
                background_color: Color::rgba(0.0, 0.0, 0.0, 0.9).into(),
                ..default()
            },
            OnScreen,
        ))
        .with_children(|parent| {
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
                    // winner text
                    parent.spawn(
                        TextBundle::from_section(
                            if is_self { "You won!" } else { "You lost!" }, // TODO: show winner name if we lost?
                            TextStyle {
                                font: asset_server.load("fonts/Lato-BlackItalic.ttf"),
                                font_size: 112.0,
                                color: Color::WHITE,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(30.0)),
                            ..default()
                        }),
                    );

                    let button_style = Style {
                        width: Val::Px(274.0),
                        height: Val::Px(72.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    };

                    // show play again button on the peer hosting
                    if let ServerState::Server(_) = **server_state {
                        parent.spawn((
                            ButtonBundle {
                                style: button_style.clone(),
                                background_color: Color::WHITE.into(),
                                image: asset_server.load("textures/buttons/play_again.png").into(),
                                ..default()
                            },
                            ButtonAction::PlayAgain,
                        ));
                    }

                    parent.spawn((
                        ButtonBundle {
                            style: button_style,
                            background_color: Color::WHITE.into(),
                            image: asset_server.load("textures/buttons/main_menu.png").into(),
                            ..default()
                        },
                        ButtonAction::Quit,
                    ));
                });
        });
}

/// Handles button presses.
pub fn handle_action(
    interaction_query: Query<&ButtonAction, (Changed<Interaction>, With<Button>)>,
    mut restart_events: EventWriter<RestartGame>,
    mouse: Res<Input<MouseButton>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut screen_state: ResMut<NextState<ScreenState>>,
    mut server_state: ResMut<NextState<ServerState>>,
    mut game_screen_state: ResMut<NextState<GameScreenState>>,
) {
    for menu_button_action in &interaction_query {
        if mouse.just_released(MouseButton::Left) {
            match menu_button_action {
                ButtonAction::Quit => {
                    menu_state.set(MenuState::Main);
                    screen_state.set(ScreenState::Menu);
                    server_state.set(ServerState::None);
                    game_screen_state.set(GameScreenState::Game);
                }
                ButtonAction::PlayAgain => {
                    restart_events.send(RestartGame);
                    game_screen_state.set(GameScreenState::Game);
                }
            }
        }
    }
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Win>()
            .add_systems(
                Update,
                handle_win
                    .run_if(in_state(ScreenState::Game))
                    .run_if(resource_exists::<MatchboxSocket<SingleChannel>>()),
            )
            .add_systems(Update, handle_action.run_if(in_state(GameScreenState::Win)))
            .add_systems(OnExit(GameScreenState::Win), despawn_screen::<OnScreen>);
    }
}
