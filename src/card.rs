//! Card struct and spawn handling.

use crate::game_ui::board::OnScreen;
use crate::game_ui::hand::HandCard;
use crate::{
    game_ui::board::{DiscardCard, DISCARD_PILE_POS, DRAW_PILE_POS, HAND_POS},
    ScreenState,
};
use bevy::prelude::{Plugin as BevyPlugin, *};

pub const CARD_SIZE: Vec2 = Vec2::new(156.0, 204.0);
pub const CARD_ANIMATION_SPEED: f32 = 7.0;

/// Card sprite component.
#[derive(Component)]
pub struct CardSprite(pub Card);

/// Place where card should be.
pub enum CardType {
    /// Local player's hand.
    Hand,
    /// Discard pile.
    Discard,
}

/// Location for card to spawn in.
pub enum CardPosition {
    /// Hand position.
    Hand,
    /// Draw pile position.
    Draw,
    /// Positioned above the screen (for animation into discard pile).
    OpponentDiscard(usize),
    /// Discard pile position.
    Discard(usize),
    // Custom position.
    // Custom(Vec3),
}

/// Event for spawning a card.
#[derive(Event)]
pub struct SpawnCard {
    pub card: Card,
    pub position: CardPosition,
    pub card_type: CardType,
}

/// Card color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardColor {
    Red,
    Yellow,
    Green,
    Blue,
    Wild,
}

impl Into<u8> for CardColor {
    fn into(self) -> u8 {
        match self {
            CardColor::Red => 0,
            CardColor::Yellow => 1,
            CardColor::Green => 2,
            CardColor::Blue => 3,
            CardColor::Wild => 4,
        }
    }
}

impl From<u8> for CardColor {
    fn from(value: u8) -> Self {
        match value {
            0 => CardColor::Red,
            1 => CardColor::Yellow,
            2 => CardColor::Green,
            3 => CardColor::Blue,
            4 => CardColor::Wild,
            _ => panic!("Invalid card color"),
        }
    }
}

/// Card value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardValue {
    Zero,
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Skip,
    Reverse,
    DrawTwo,
}

/// Card struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Card {
    pub color: CardColor,
    pub value: CardValue,
    /// A number used to differentiate cards of the same color and value
    pub iteration: u8,
}

impl Card {
    /// Creates a new card with the given color, value, and iteration.
    pub fn new(color: CardColor, value: CardValue, iteration: u8) -> Self {
        Self {
            color,
            value,
            iteration,
        }
    }

    /// Returns true if the card can be played on the given card.
    ///
    /// Checks if the colors or values match, or if the card is a wild,
    /// but if the card being played is a wild, it returns false, since
    /// we need to wait until the player chooses a color
    pub fn can_play_on(&self, card: &Card) -> bool {
        (self.color == card.color || self.value == card.value || self.color == CardColor::Wild)
            && card.color != CardColor::Wild
    }

    // pub fn text(&self, font: Handle<Font>) -> Text2dBundle {
    //     Text2dBundle {
    //         text: Text::from_section(
    //             match self.value {
    //                 CardValue::Zero => "0",
    //                 CardValue::One => "1",
    //                 CardValue::Two => "2",
    //                 CardValue::Three => "3",
    //                 CardValue::Four => "4",
    //                 CardValue::Five => "5",
    //                 CardValue::Six => "6",
    //                 CardValue::Seven => "7",
    //                 CardValue::Eight => "8",
    //                 CardValue::Nine => "9",
    //                 CardValue::Skip => "Skip",
    //                 CardValue::Reverse => "Reverse",
    //                 CardValue::DrawTwo => "+2",
    //             },
    //             TextStyle {
    //                 font,
    //                 font_size: 40.0,
    //                 color: Color::WHITE,
    //             },
    //         )
    //         .with_alignment(TextAlignment::Center),
    //         transform: Transform::from_xyz(
    //             -CARD_SIZE.x / 2.0 + 20.0,
    //             CARD_SIZE.y / 2.0 - 20.0,
    //             0.0005,
    //         ),
    //         ..default()
    //     }
    // }

    /// Returns a sprite bundle for the card.
    pub fn sprite(&self, position: Vec3, asset_server: &Res<AssetServer>) -> SpriteBundle {
        let file_name = {
            let value = match self.value {
                CardValue::Zero => "0",
                CardValue::One => "1",
                CardValue::Two => "2",
                CardValue::Three => "3",
                CardValue::Four => "4",
                CardValue::Five => "5",
                CardValue::Six => "6",
                CardValue::Seven => "7",
                CardValue::Eight => "8",
                CardValue::Nine => "9",
                CardValue::Skip => "skip",
                CardValue::Reverse => "rev",
                CardValue::DrawTwo => "draw2",
            };
            let color = match self.color {
                CardColor::Red => "red",
                CardColor::Yellow => "yellow",
                CardColor::Green => "green",
                CardColor::Blue => "blue",
                CardColor::Wild => "wild",
            };
            format!("{}{}", color, value)
        };
        SpriteBundle {
            sprite: Sprite {
                // color: match self.color {
                //     CardColor::Red => Color::RED,
                //     CardColor::Yellow => Color::YELLOW,
                //     CardColor::Green => Color::GREEN,
                //     CardColor::Blue => Color::BLUE,
                // },
                custom_size: Some(CARD_SIZE),
                ..default()
            },
            texture: asset_server.load(format!("textures/cards/{file_name}.png")),
            transform: Transform::from_translation(position),
            ..default()
        }
    }
}

impl Into<u8> for Card {
    // returns a number from 0 to 103
    fn into(self) -> u8 {
        let color = match self.color {
            CardColor::Red => 0,
            CardColor::Yellow => 1,
            CardColor::Green => 2,
            CardColor::Blue => 3,
            CardColor::Wild => return 104 + self.iteration,
        };
        let value = match self.value {
            CardValue::Zero => 0,
            CardValue::One => 1,
            CardValue::Two => 2,
            CardValue::Three => 3,
            CardValue::Four => 4,
            CardValue::Five => 5,
            CardValue::Six => 6,
            CardValue::Seven => 7,
            CardValue::Eight => 8,
            CardValue::Nine => 9,
            CardValue::Skip => 10,
            CardValue::Reverse => 11,
            CardValue::DrawTwo => 12,
        };
        (color * 13 + value) + (self.iteration - 1) * 52
    }
}

impl From<u8> for Card {
    fn from(value: u8) -> Self {
        if value >= 104 {
            return Self {
                color: CardColor::Wild,
                value: CardValue::Seven,
                iteration: value - 104,
            };
        }
        let (value, iteration) = if value <= 51 {
            (value, 1)
        } else {
            (value - 52, 2)
        };
        let color = match value / 13 {
            0 => CardColor::Red,
            1 => CardColor::Yellow,
            2 => CardColor::Green,
            3 => CardColor::Blue,
            _ => unreachable!(),
        };
        let value = match value % 13 {
            0 => CardValue::Zero,
            1 => CardValue::One,
            2 => CardValue::Two,
            3 => CardValue::Three,
            4 => CardValue::Four,
            5 => CardValue::Five,
            6 => CardValue::Six,
            7 => CardValue::Seven,
            8 => CardValue::Eight,
            9 => CardValue::Nine,
            10 => CardValue::Skip,
            11 => CardValue::Reverse,
            12 => CardValue::DrawTwo,
            _ => unreachable!(),
        };
        Card::new(color, value, iteration)
    }
}

/// Recieves card spawn events and spawns cards.
fn handle_spawn_card(
    mut events: EventReader<SpawnCard>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for event in events.read() {
        let position = match event.position {
            CardPosition::Draw => DRAW_PILE_POS,
            CardPosition::OpponentDiscard(count) => {
                Vec3::new(0.0, -300.0, (count + 1) as f32 * 0.01)
            }
            CardPosition::Discard(count) => {
                let mut position = DISCARD_PILE_POS;
                position.z = (count + 1) as f32 * 0.01;
                position
            }
            CardPosition::Hand => HAND_POS,
            // CardPosition::Custom(pos) => pos,
        };
        let mut entity = commands.spawn((
            event.card.sprite(position, &asset_server),
            CardSprite(event.card),
            OnScreen,
        ));
        match event.card_type {
            CardType::Hand => entity.insert(HandCard::new(event.card)),
            CardType::Discard => entity.insert(DiscardCard),
        };
    }
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCard>().add_systems(
            Update,
            handle_spawn_card.run_if(in_state(ScreenState::Game)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::super::deck::Deck;
    use super::*;

    /// Ensures that all cards can be serialized and then deserialized back to themselves.
    #[test]
    fn test_card_serialization() {
        let deck = Deck::new();
        for card in deck.cards {
            let serialized: u8 = card.into();
            let deserialized = Card::from(serialized);
            assert_eq!(card, deserialized);
        }
    }
}
