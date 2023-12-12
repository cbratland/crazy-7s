//! The cards in main player's hand.

use crate::card::{Card, CardColor, CARD_ANIMATION_SPEED, CARD_SIZE};
use crate::deck::{DiscardCards, MainPlayer};
use crate::game_ui::board::{DiscardCard, DrawPile, HAND_POS};
use crate::info::GameInfo;
use crate::network::PlayCard;
use crate::screens::wild::Wild;
use crate::{GameScreenState, ScreenState, WorldCoords};
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_matchbox::prelude::*;

/// Offset for hovering cards in hand.
const HOVER_OFFSET: f32 = 20.0;

/// Currently hovering component.
#[derive(Component)]
pub struct Hovering;

/// Card in player's hand component.
#[derive(Component)]
pub struct HandCard {
    card: Card,
}

impl HandCard {
    pub fn new(card: Card) -> Self {
        Self { card }
    }
}

/// Handles clicking on a card in the player's hand.
fn handle_card_click(
    mut cards: Query<(Entity, &HandCard, &mut Transform), With<Hovering>>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut discard_pile: ResMut<DiscardCards>,
    mut play_events: EventWriter<PlayCard>,
    mut wild_events: EventWriter<Wild>,
    mut player: ResMut<MainPlayer>,
    game_info: Res<GameInfo>,
    mouse: Res<Input<MouseButton>>,
    mut commands: Commands,
) {
    if mouse.just_released(MouseButton::Left) {
        // ensure it's the player's turn
        let Some(own_id) = socket.id() else { return; };
        if game_info.current_player.map_or(true, |id| own_id != id) {
            return;
        };

        let Some((entity, HandCard { card }, mut transform)) = cards.iter_mut().next() else { return; };

        // ensure card can be played
        if let Some(top_card) = discard_pile.cards.last() {
            if !card.can_play_on(top_card) {
                return;
            }
        }

        // remove card from player's hand
        let index = player
            .cards
            .iter()
            .position(|x| *x == *card)
            .expect("invalid card id");
        player.cards.remove(index);

        // add card to discard pile card count and set z position to top
        discard_pile.cards.push(*card);
        transform.translation.z = (discard_pile.cards.len() as f32 + 1.0) * 0.01;

        // mark card entity as discarded
        commands.entity(entity).remove::<Hovering>();
        commands.entity(entity).remove::<HandCard>();
        commands.entity(entity).insert(DiscardCard);

        // send card played event to game flow system
        // card_events.send(PlayCard::new(*card, 0));

        if card.color == CardColor::Wild {
            wild_events.send(Wild);
        }

        play_events.send(PlayCard(*card));
    }
}

/// Moves cards to correct position in the player's hand.
fn animate_hand_cards(
    mut cards: Query<(&mut Transform, &HandCard)>,
    player: Res<MainPlayer>,
    time: Res<Time>,
) {
    let card_speed = CARD_ANIMATION_SPEED * time.delta_seconds();
    let card_count = player.cards.len();
    let center_idx = (card_count as f32 - 1.0) / 2.0;
    let spacing = if card_count <= 7 {
        CARD_SIZE.x / 2.0
    } else {
        CARD_SIZE.x / (2.0 + (card_count - 7) as f32 / 4.0)
    };

    for (mut transform, HandCard { card }) in &mut cards {
        // find real index in player cards
        let Some(index) = player
            .cards
            .iter()
            .position(|x| *x == *card) else { continue; };

        let x_offset = -spacing * (center_idx - index as f32);
        let target = Vec3::new(x_offset, 0.0, 0.0);
        let mut target = HAND_POS + target;
        target.z = 0.01 * index as f32;
        let origin = transform.translation;
        let distance = target - origin;
        if distance.length() < 0.01 {
            continue;
        }
        transform.translation += (target - origin) * card_speed;
    }
}

/// Detects when the mouse is hovering over a card or the draw pile.
fn detect_hover(
    cards: Query<(Entity, &Transform), Or<(With<HandCard>, With<DrawPile>)>>,
    coords: Res<WorldCoords>,
    mut commands: Commands,
) {
    let WorldCoords(coords) = *coords;
    let mut top_entity: Option<Entity> = None;
    let mut top_z = -1.0;
    // check if card is hovered
    for (card, transform) in &cards {
        if coords.x > transform.translation.x - CARD_SIZE.x / 2.0
            && coords.x < transform.translation.x + CARD_SIZE.x / 2.0
            && coords.y > transform.translation.y - CARD_SIZE.y / 2.0
            && coords.y < transform.translation.y + CARD_SIZE.y / 2.0
            && transform.translation.z > top_z
        {
            if let Some(entity) = top_entity {
                commands.entity(entity).remove::<Hovering>();
            }
            top_entity = Some(card);
            top_z = transform.translation.z;
        } else {
            commands.entity(card).remove::<Hovering>();
        }
    }
    if let Some(entity) = top_entity {
        commands.entity(entity).insert(Hovering);
    }
}

/// Moves cards in hand up slightly when hovered.
fn animate_card_hover(
    // hand: Query<&GlobalTransform, With<PlayerHand>>,
    mut cards: Query<&mut Transform, (With<HandCard>, With<Hovering>)>,
    time: Res<Time>,
) {
    let card_speed = CARD_ANIMATION_SPEED * time.delta_seconds();
    let target = HAND_POS.y + HOVER_OFFSET;

    for mut transform in &mut cards {
        let current = transform.translation.y;
        let distance = target - current;
        if distance < 0.1 {
            continue;
        }
        transform.translation.y += distance * card_speed;
    }
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayCard>()
            .add_systems(
                Update,
                animate_hand_cards.run_if(in_state(ScreenState::Game)),
            )
            .add_systems(
                Update,
                (handle_card_click, detect_hover, animate_card_hover)
                    .run_if(in_state(ScreenState::Game))
                    .run_if(in_state(GameScreenState::Game))
                    .run_if(resource_exists::<MatchboxSocket<SingleChannel>>()),
            );
    }
}
