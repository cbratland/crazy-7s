//! The overall deck of cards, discard pile, and player card resources.

use crate::card::{Card, CardColor, CardValue};
use bevy::prelude::{Plugin as BevyPlugin, *};

/// Deck of cards.
#[derive(Resource, Debug, Clone)]
pub struct Deck {
    pub cards: Vec<Card>,
}

impl Deck {
    /// The default cards in the deck.
    fn default_cards() -> Vec<Card> {
        let mut cards = Vec::new();
        // add regular deck (without 7s)
        for color in [
            CardColor::Red,
            CardColor::Yellow,
            CardColor::Green,
            CardColor::Blue,
        ] {
            for value in [
                CardValue::Zero,
                CardValue::One,
                CardValue::Two,
                CardValue::Three,
                CardValue::Four,
                CardValue::Five,
                CardValue::Six,
                CardValue::Eight,
                CardValue::Nine,
                CardValue::Skip,
                CardValue::Reverse,
                CardValue::DrawTwo,
            ] {
                cards.push(Card::new(color, value, 1));
                cards.push(Card::new(color, value, 2));
            }
        }
        // add four wild cards
        for i in 0..4 {
            cards.push(Card::new(CardColor::Wild, CardValue::Seven, i));
        }
        cards
    }

    /// Creates a new deck of cards with the default cards.
    pub fn new() -> Self {
        let cards = Self::default_cards();
        Self { cards }
    }

    // Resets the deck to the default cards.
    // pub fn reset(&mut self) {
    //     self.cards = Self::default_cards();
    // }

    /// Shuffles the deck.
    pub fn shuffle(&mut self) {
        use rand::seq::SliceRandom;
        use rand::thread_rng;
        self.cards.shuffle(&mut thread_rng());
    }

    /// Returns the order of the cards in the deck.
    pub fn get_card_order(&self) -> Vec<u8> {
        self.cards.iter().map(|c| (*c).into()).collect()
    }

    /// Loads the deck from the given order of cards.
    pub fn load_from(&mut self, order: &[u8]) {
        self.cards = order.iter().map(|v| Card::from(*v)).collect();
    }

    /// Draws the given number of cards from the deck.
    pub fn draw(&mut self, n: i32) -> Vec<Card> {
        let mut cards = Vec::new();
        for _ in 0..n {
            if let Some(card) = self.cards.pop() {
                cards.push(card);
            }
        }
        cards
    }

    /// Returns `true` if the deck has no cards left.
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
}

impl FromWorld for Deck {
    fn from_world(_: &mut World) -> Self {
        let mut deck = Self::new();
        deck.shuffle();
        return deck;
    }
}

/// The cards that have been discarded.
#[derive(Resource, Default)]
pub struct DiscardCards {
    pub cards: Vec<Card>,
}

/// The main player's cards.
#[derive(Resource, Default)]
pub struct MainPlayer {
    pub cards: Vec<Card>,
}

impl MainPlayer {
    pub fn reset(&mut self) {
        self.cards.clear();
    }
}

/// Initializes deck and main player cards.
fn setup(mut commands: Commands) {
    commands.init_resource::<Deck>();
    commands.insert_resource(DiscardCards::default());
    commands.insert_resource(MainPlayer::default());
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}
