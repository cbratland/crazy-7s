//! Game info and opponents resources.

use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_matchbox::matchbox_socket::PeerId;

#[derive(Debug)]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

#[derive(Resource)]
pub struct GameInfo {
    pub current_player: Option<PeerId>,
    pub order: Vec<PeerId>,
    pub direction: Direction,
}

impl FromWorld for GameInfo {
    fn from_world(_: &mut World) -> Self {
        GameInfo {
            current_player: None,
            order: Vec::new(),
            direction: Direction::Clockwise,
        }
    }
}

impl GameInfo {
    pub fn reset(&mut self) {
        self.current_player = None;
        self.order = Vec::new();
        self.direction = Direction::Clockwise;
    }

    // moves to the next player in the order and returns the new current player
    pub fn advance_turn(&mut self) -> Option<PeerId> {
        let next_player = match self.current_player {
            Some(current_player) => {
                let current_index = self
                    .order
                    .iter()
                    .position(|&p| p == current_player)
                    .unwrap();
                let next_index = match self.direction {
                    Direction::Clockwise => current_index + 1,
                    Direction::CounterClockwise => current_index + self.order.len() - 1,
                } % self.order.len();
                Some(self.order[next_index])
            }
            None => None,
        };
        self.current_player = next_player;
        next_player
    }

    pub fn swap_direction(&mut self) {
        self.direction = match self.direction {
            Direction::Clockwise => Direction::CounterClockwise,
            Direction::CounterClockwise => Direction::Clockwise,
        }
    }
}

/// Opponent component.
#[derive(Clone, Debug)]
pub struct Opponent {
    pub id: PeerId,
    pub name: String,
    pub card_count: usize,
}

impl Opponent {
    pub fn new(id: PeerId, name: String, card_count: usize) -> Self {
        Self {
            id,
            name,
            card_count,
        }
    }
}

/// Opponent list resource.
#[derive(Resource)]
pub struct Opponents(pub Vec<Opponent>);

/// Initializes the game info and discard pile resource.
fn setup(mut commands: Commands) {
    commands.init_resource::<GameInfo>();
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}
