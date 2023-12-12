//! Draw and discard piles.

use crate::card::{
    Card, CardColor, CardPosition, CardSprite, CardType, CardValue, SpawnCard, CARD_ANIMATION_SPEED,
};
use crate::deck::{Deck, DiscardCards, MainPlayer};
use crate::game_ui::hand::Hovering;
use crate::info::GameInfo;
use crate::network::DrawCard;
use crate::network::ServerState;
use crate::GameScreenState;
use crate::{despawn_screen, ScreenState};
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_matchbox::prelude::*;

/// Position of the draw pile.
pub const DRAW_PILE_POS: Vec3 = Vec3::new(-92.0, 0.0, 0.01);
/// Position of the discard pile.
pub const DISCARD_PILE_POS: Vec3 = Vec3::new(92.0, 0.0, 0.01);
/// Position of the player's hand.
pub const HAND_POS: Vec3 = Vec3::new(0.0, -250.0, 0.0);

/// Component for the draw pile.
#[derive(Component)]
pub struct DrawPile;

/// Component for the discard pile.
#[derive(Component)]
pub struct DiscardPile;

/// Indicates that the card is meant to be in the discard pile.
#[derive(Component)]
pub struct DiscardCard;

/// Indicates that the component bundle is for this screen.
#[derive(Component)]
pub struct OnScreen;

/// Indicates the bundle's associated button action.
#[derive(Component)]
enum ButtonAction {
    BackToMenu,
}

/// Draws piles and menu button.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // back to menu button
    commands.spawn((
        ButtonBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                width: Val::Px(46.0),
                height: Val::Px(36.0),
                margin: UiRect::all(Val::Px(20.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            background_color: Color::WHITE.into(),
            image: asset_server.load("textures/buttons/menu.png").into(),
            ..default()
        },
        ButtonAction::BackToMenu,
        OnScreen,
    ));

    // draw pile
    let mut position = DRAW_PILE_POS;
    position.z = 0.0;
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                // color: Color::WHITE,
                custom_size: Some(Vec2::new(156.0, 218.0)),
                ..default()
            },
            texture: asset_server.load("textures/drawpile.png"),
            transform: Transform::from_translation(position),
            ..default()
        },
        DrawPile,
        OnScreen,
    ));

    // set discard pile position
    position = DISCARD_PILE_POS;
    position.z = 0.0;
    commands.spawn((
        GlobalTransform::default(),
        Transform::from_translation(position),
        DiscardPile,
        OnScreen,
    ));
}

/// Handles button presses.
fn handle_menu_action(
    interaction_query: Query<&ButtonAction, (Changed<Interaction>, With<Button>)>,
    mut screen_state: ResMut<NextState<ScreenState>>,
    mut server_state: ResMut<NextState<ServerState>>,
    mouse: Res<Input<MouseButton>>,
) {
    for menu_button_action in &interaction_query {
        if mouse.just_released(MouseButton::Left) {
            match menu_button_action {
                ButtonAction::BackToMenu => {
                    screen_state.set(ScreenState::Menu);
                    server_state.set(ServerState::None);
                }
            }
        }
    }
}

/// Spawns a new card when the draw pile is clicked.
fn draw_card(
    // interaction_query: Query<&Interaction, (Changed<Interaction>, With<Button>)>,
    pile: Query<Entity, (With<DrawPile>, With<Hovering>)>,
    mut spawn_events: EventWriter<SpawnCard>,
    mut draw_events: EventWriter<DrawCard>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut player: ResMut<MainPlayer>,
    mut deck: ResMut<Deck>,
    discard_pile: Res<DiscardCards>,
    mouse: Res<Input<MouseButton>>,
    game_info: Res<GameInfo>,
) {
    // ensure that draw pile is being hovered over
    if pile.iter().next().is_none() {
        return;
    };
    if mouse.just_released(MouseButton::Left) {
        // if top card is an uncolored wild card, don't allow drawing (we need to wait until color is chosen)
        if let Some(top_card) = discard_pile.cards.last() {
            if top_card.color == CardColor::Wild {
                return;
            }
        }

        // ensure it's the player's turn
        let Some(own_id) = socket.id() else { return; };
        if game_info.current_player.map_or(true, |id| own_id != id) {
            return;
        };

        let Some(card) = deck.draw(1).iter().next().copied() else {
           	println!("No cards left in deck");
           	return;
        };
        player.cards.push(card);
        spawn_events.send(SpawnCard {
            card,
            position: CardPosition::Draw,
            card_type: CardType::Hand,
        });
        draw_events.send(DrawCard);
    };
}

/// Moves the cards from discard pile into draw pile and shuffles if the draw pile is empty.
// TODO: make this not disappear the card underneath immediately if we play a card with no cards in the draw pile
fn shuffle_discard_pile(
    mut discard_pile: ResMut<DiscardCards>,
    mut discard_cards: Query<(Entity, &CardSprite), With<DiscardCard>>,
    mut deck: ResMut<Deck>,
    mut commands: Commands,
) {
    if deck.is_empty() {
        let len = discard_pile.cards.len();
        if len <= 1 {
            return;
        }
        let mut cards: Vec<Card> = discard_pile.cards.drain(..len - 1).collect();
        let top_card = discard_pile.cards[0];
        // despawn cards we removed from discard pile
        for (entity, CardSprite(card)) in discard_cards.iter_mut() {
            if *card == top_card {
                continue;
            }
            commands.entity(entity).despawn_recursive();
        }
        // reset wild cards
        for mut card in cards.iter_mut() {
            if card.value == CardValue::Seven {
                card.color = CardColor::Wild;
            }
        }
        deck.cards.append(&mut cards);
        deck.shuffle();
    }
}

/// Moves discarded cards to the discard pile.
fn animate_card_discard(
    discard_pile: Query<&GlobalTransform, With<DiscardPile>>,
    mut cards: Query<(Entity, &mut Transform), With<DiscardCard>>,
    time: Res<Time>,
) {
    let card_speed = CARD_ANIMATION_SPEED * time.delta_seconds();
    let target = discard_pile.single().compute_transform().translation;

    for (_, mut transform) in &mut cards {
        let mut origin = transform.translation;
        origin.z = 0.0;
        let distance = target - origin;
        if distance.length() < 0.1 {
            continue;
        }
        transform.translation += distance * card_speed;
    }
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(ScreenState::Game), setup)
            .add_systems(OnExit(ScreenState::Game), despawn_screen::<OnScreen>)
            .add_systems(
                Update,
                (handle_menu_action, animate_card_discard).run_if(in_state(ScreenState::Game)),
            )
            // systems disabled if a different game screen is shown (winner/wild choose)
            .add_systems(
                Update,
                (draw_card, shuffle_discard_pile)
                    .run_if(in_state(ScreenState::Game))
                    .run_if(in_state(GameScreenState::Game))
                    .run_if(resource_exists::<MatchboxSocket<SingleChannel>>()),
            );
    }
}
