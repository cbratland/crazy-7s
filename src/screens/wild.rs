//! Wild card color selection screen.

use crate::{
    card::{CardColor, CardType, SpawnCard},
    deck::DiscardCards,
    despawn_screen,
    network::WildColor,
    GameScreenState, ScreenState,
};
use bevy::prelude::{Plugin as BevyPlugin, *};

/// Event posted when a wild card is played by the local player.
#[derive(Event)]
pub struct Wild;

/// Indicates that the component bundle is for this screen.
#[derive(Component)]
pub struct OnScreen;

/// Indicates the bundle's associated button action.
#[derive(Component)]
pub enum ButtonAction {
    Red,
    Yellow,
    Green,
    Blue,
}

impl ToString for ButtonAction {
    fn to_string(&self) -> String {
        match self {
            ButtonAction::Red => "red".to_string(),
            ButtonAction::Yellow => "yellow".to_string(),
            ButtonAction::Green => "green".to_string(),
            ButtonAction::Blue => "blue".to_string(),
        }
    }
}

/// Draws wild color selection screen when wild card event is received.
fn handle_wild(
    mut events: EventReader<Wild>,
    mut game_screen_state: ResMut<NextState<GameScreenState>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    if events.read().next().is_none() {
        return;
    };
    game_screen_state.set(GameScreenState::WildColor);

    // draw wild screen
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
                    let button_style = Style {
                        width: Val::Px(222.0),
                        height: Val::Px(78.0),
                        margin: UiRect::all(Val::Px(20.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    };
                    let button_text_style = TextStyle {
                        font: asset_server.load("fonts/Lato-BlackItalic.ttf"),
                        font_size: 50.0,
                        color: Color::BLACK,
                    };

                    for action in [
                        ButtonAction::Red,
                        ButtonAction::Yellow,
                        ButtonAction::Green,
                        ButtonAction::Blue,
                    ] {
                        let title = action.to_string();
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: Color::WHITE.into(),
                                    ..default()
                                },
                                action,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    title,
                                    button_text_style.clone(),
                                ));
                            });
                    }
                });
        });
}

/// Handles button presses.
pub fn handle_action(
    interaction_query: Query<&ButtonAction, (Changed<Interaction>, With<Button>)>,
    mut discard_pile: ResMut<DiscardCards>,
    mut spawn_events: EventWriter<SpawnCard>,
    mut wild_events: EventWriter<WildColor>,
    mut game_screen_state: ResMut<NextState<GameScreenState>>,
    mouse: Res<Input<MouseButton>>,
) {
    for menu_button_action in &interaction_query {
        if mouse.just_released(MouseButton::Left) {
            let card_color = match menu_button_action {
                ButtonAction::Red => CardColor::Red,
                ButtonAction::Yellow => CardColor::Yellow,
                ButtonAction::Green => CardColor::Green,
                ButtonAction::Blue => CardColor::Blue,
            };

            // add the colored 7 to top of discard pile
            let mut new_card = discard_pile.cards.last().unwrap().clone();
            new_card.color = card_color;
            discard_pile.cards.push(new_card);

            // spawn a new seven on top of the discard pile with proper color
            spawn_events.send(SpawnCard {
                card: new_card,
                position: crate::card::CardPosition::Discard(discard_pile.cards.len()),
                card_type: CardType::Discard,
            });

            wild_events.send(WildColor(card_color));

            game_screen_state.set(GameScreenState::Game);
        }
    }
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<Wild>()
            .add_systems(Update, handle_wild.run_if(in_state(ScreenState::Game)))
            .add_systems(
                Update,
                handle_action.run_if(in_state(GameScreenState::WildColor)),
            )
            .add_systems(
                OnExit(GameScreenState::WildColor),
                despawn_screen::<OnScreen>,
            );
    }
}
