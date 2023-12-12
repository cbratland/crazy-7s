//! Peer to peer communication and game events.

use crate::{
    card::{Card, CardColor, CardPosition, CardType, CardValue, SpawnCard},
    deck::{Deck, DiscardCards, MainPlayer},
    game_ui::board::DiscardCard,
    game_ui::hand::HandCard,
    info::{GameInfo, Opponent, Opponents},
    menu::MenuState,
    screens::win::Win,
    GameScreenState, ScreenState, Username,
};
use bevy::{
    prelude::{Plugin as BevyPlugin, *},
    utils::{HashMap, Uuid},
};
use bevy_matchbox::prelude::*;

/// Server state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, States)]
pub enum ServerState {
    #[default]
    None,
    Server(u16),
    Client(u16),
}

/// Storage of names for connected peers.
#[derive(Resource)]
pub struct PeerNames(pub HashMap<PeerId, String>);

/// Socket event, which corresponds to one byte.
#[derive(PartialEq, Eq)]
pub enum SocketEvent {
    Start,
    Draw,
    Play,
    Restart,
    Name,
    Wild,
}

impl Into<u8> for SocketEvent {
    fn into(self) -> u8 {
        match self {
            Self::Start => 0,
            Self::Draw => 1,
            Self::Play => 2,
            Self::Restart => 3,
            Self::Name => 4,
            Self::Wild => 5,
        }
    }
}

pub enum SocketEventInitError {
    InvalidByte,
}

impl TryFrom<u8> for SocketEvent {
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Start),
            1 => Ok(Self::Draw),
            2 => Ok(Self::Play),
            3 => Ok(Self::Restart),
            4 => Ok(Self::Name),
            5 => Ok(Self::Wild),
            _ => Err(SocketEventInitError::InvalidByte),
        }
    }

    type Error = SocketEventInitError;
}

/// Start game event triggered by host.
#[derive(Event)]
pub struct StartGame {
    pub order: Vec<PeerId>,
    pub restart: bool,
}

/// Draw card event triggered by a client.
#[derive(Event)]
pub struct DrawCard;

/// Play card event triggered by a client.
#[derive(Event)]
pub struct PlayCard(pub Card);

/// Restart game event.
#[derive(Event)]
pub struct RestartGame;

/// Wild color selection event.
///
/// This event is triggered by the host after a wild card is played and the color is selected.
#[derive(Event)]
pub struct WildColor(pub CardColor);

/// Initializes the peer names hashmap.
fn setup(mut commands: Commands) {
    commands.insert_resource(PeerNames(HashMap::new()));
}

/// Receives messages from the network and handles peer connections.
fn receive_messages(
    hand_cards: Query<Entity, With<HandCard>>,
    discard_cards: Query<Entity, With<DiscardCard>>,
    mut discard_pile: ResMut<DiscardCards>,
    mut spawn_events: EventWriter<SpawnCard>,
    mut win_events: EventWriter<Win>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut main_player: ResMut<MainPlayer>,
    mut game_info: ResMut<GameInfo>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut screen_state: ResMut<NextState<ScreenState>>,
    mut game_screen_state: ResMut<NextState<GameScreenState>>,
    mut peer_names: ResMut<PeerNames>,
    mut opponents: ResMut<Opponents>,
    mut deck: ResMut<Deck>,
    username: Res<Username>,
    mut commands: Commands,
) {
    // Check for new connections
    match socket.try_update_peers() {
        Ok(result) => {
            for (peer, state) in result {
                match state {
                    PeerState::Connected => {
                        info!("Peer joined: {peer}");
                        // send our username to the peer
                        let mut packet = username.0.as_bytes().to_vec();
                        packet.insert(0, SocketEvent::Name.into());
                        socket.send(packet.into_boxed_slice(), peer);
                    }
                    PeerState::Disconnected => {
                        info!("Peer left: {peer}");
                        // remove stored peer name
                        peer_names.0.remove(&peer);
                    }
                }
            }
        }
        Err(e) => {
            error!("Error updating peers: {e:?}");
        }
    }

    // Accept any messages incoming
    for (peer, packet) in socket.receive() {
        let Some(event_code) = packet.first() else { return; };
        let Ok(event): Result<SocketEvent, _> = (*event_code).try_into() else {
        	error!("Received invalid event code: {event_code}");
        	return;
        };
        match event {
            SocketEvent::Start | SocketEvent::Restart => {
                // reset the game start before starting the game if we're restarting
                if event == SocketEvent::Restart {
                    reset_game_state(
                        &discard_cards,
                        &hand_cards,
                        &mut game_screen_state,
                        &mut discard_pile,
                        &mut main_player,
                        &mut opponents,
                        &mut game_info,
                        &mut commands,
                    );
                }

                // load player order
                // first byte is the number of players, then 16 bytes for each player id
                let player_count = packet[1];
                let mut order: Vec<PeerId> = Vec::new();
                let mut current_pid: [u8; 16] = [0; 16];
                let mut packet_pos = 2;
                for _ in 0..player_count {
                    for i in 0..16 {
                        if packet_pos >= packet.len() {
                            error!("Invalid start game packet: ran out of bytes.");
                            return;
                        }
                        current_pid[i] = packet[packet_pos];
                        packet_pos += 1;
                    }
                    order.push(PeerId(Uuid::from_bytes(current_pid)));
                }

                // load opponents
                let own_pid = socket.id().expect("server should assign us a peer id");
                opponents.0 = order
                    .iter()
                    .filter_map(|pid| {
                        // skip our own id
                        if *pid == own_pid {
                            None
                        } else {
                            let name = peer_names
                                .0
                                .get(pid)
                                .cloned()
                                .unwrap_or_else(|| String::from("Unknown"));
                            Some(Opponent::new(*pid, name, 5))
                        }
                    })
                    .collect();

                // set game state info
                game_info.order = order;
                game_info.current_player = game_info.order.first().copied();

                // load deck from order
                // the remaining bytes should be the deck, with a byte for each card
                deck.load_from(&packet[packet_pos..]);

                initialize_game_start(
                    &own_pid,
                    &mut spawn_events,
                    &mut game_info,
                    &mut main_player,
                    &mut deck,
                    &mut discard_pile,
                    &mut screen_state,
                    &mut menu_state,
                )
            }
            SocketEvent::Draw => {
                deck.draw(1);

                // increment card count for opponent
                for opponent in opponents.0.iter_mut() {
                    if opponent.id == peer {
                        opponent.card_count += 1;
                        break;
                    }
                }

                game_info.advance_turn();
            }
            SocketEvent::Play => {
                let card = Card::from(packet[1]);

                // add to discard pile
                discard_pile.cards.push(card);

                // spawn card
                spawn_events.send(SpawnCard {
                    card,
                    position: CardPosition::OpponentDiscard(discard_pile.cards.len()),
                    card_type: CardType::Discard,
                });

                // decrement card count for opponent
                for opponent in opponents.0.iter_mut() {
                    if opponent.id == peer {
                        opponent.card_count -= 1;
                        // check for win
                        if opponent.card_count == 0 {
                            win_events.send(Win(opponent.id));
                        }
                        break;
                    }
                }

                game_info.advance_turn();

                handle_card_effect(
                    &card,
                    &peer,
                    &mut spawn_events,
                    &mut socket,
                    &mut game_info,
                    &mut main_player,
                    &mut opponents,
                    &mut deck,
                )
            }
            SocketEvent::Name => {
                // update peer names hashmap
                let name = String::from_utf8_lossy(&packet[1..]);
                peer_names.0.insert(peer, name.to_string());
            }
            SocketEvent::Wild => {
                let card_color = CardColor::from(packet[1]);

                // add the colored wild to top of discard pile
                let mut new_card = discard_pile
                    .cards
                    .last()
                    .expect("wild card should be on top of the discard pile")
                    .clone();
                new_card.color = card_color;
                discard_pile.cards.push(new_card);

                spawn_events.send(SpawnCard {
                    card: new_card,
                    position: CardPosition::Discard(discard_pile.cards.len()),
                    card_type: CardType::Discard,
                });
            }
        }
    }
}

/// Resets the game state to the initial state.
fn reset_game_state(
    discard_cards: &Query<Entity, With<DiscardCard>>,
    hand_cards: &Query<Entity, With<HandCard>>,
    game_screen_state: &mut ResMut<NextState<GameScreenState>>,
    discard_pile: &mut ResMut<DiscardCards>,
    main_player: &mut ResMut<MainPlayer>,
    opponents: &mut ResMut<Opponents>,
    game_info: &mut ResMut<GameInfo>,
    commands: &mut Commands,
) {
    // reset game state
    game_info.reset();
    main_player.reset();
    discard_pile.cards.clear();

    // reset opponent card counts
    for opponent in opponents.0.iter_mut() {
        opponent.card_count = 5;
    }

    // despawn discard cards
    for entity in discard_cards.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // despawn hand cards
    for entity in hand_cards.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // hide win screen, show playing screen
    game_screen_state.set(GameScreenState::Game);
}

/// Loads deck, player cards, and the top discard card.
fn initialize_game_start(
    our_pid: &PeerId,
    spawn_events: &mut EventWriter<SpawnCard>,
    game_info: &mut ResMut<GameInfo>,
    main_player: &mut ResMut<MainPlayer>,
    deck: &mut ResMut<Deck>,
    discard_pile: &mut ResMut<DiscardCards>,
    screen_state: &mut ResMut<NextState<ScreenState>>,
    menu_state: &mut ResMut<NextState<MenuState>>,
) {
    // fetch cards for our hand based on order
    let our_position = game_info
        .order
        .iter()
        .position(|pid| *pid == *our_pid)
        .expect("our pid should be in the order");
    // get a new vector of the next 5 cards located at index our_position*5 from deck.cards
    let cards = deck.cards[our_position * 5..(our_position + 1) * 5].to_vec();
    main_player.cards = cards;

    // discard the cards given to the players
    deck.draw(5 * game_info.order.len() as i32);

    // spawn top card for discard pile
    let expect_msg = "complete deck should be loaded from packet";
    let mut card = deck.draw(1).first().copied().expect(expect_msg);
    if card.color == CardColor::Wild {
        // if the first card is a wild, draw another card
        discard_pile.cards.push(card);
        card = deck.draw(1).first().copied().expect(expect_msg);
    }
    discard_pile.cards.push(card);
    spawn_events.send(SpawnCard {
        card,
        position: CardPosition::Draw,
        card_type: CardType::Discard,
    });

    // spawn cards in hand
    for card in main_player.cards.iter() {
        spawn_events.send(SpawnCard {
            card: *card,
            position: CardPosition::Hand,
            card_type: CardType::Hand,
        })
    }

    // show game ui
    screen_state.set(ScreenState::Game);
    menu_state.set(MenuState::Disabled);
}

/// Performs the card effect for the given card.
///
/// Handles skips, reverses, and draw twos.
pub fn handle_card_effect(
    card: &Card,
    card_player: &PeerId,
    spawn_events: &mut EventWriter<SpawnCard>,
    socket: &mut ResMut<MatchboxSocket<SingleChannel>>,
    game_info: &mut ResMut<GameInfo>,
    main_player: &mut ResMut<MainPlayer>,
    opponents: &mut ResMut<Opponents>,
    deck: &mut ResMut<Deck>,
) {
    match card.value {
        CardValue::Skip => {
            game_info.advance_turn();
        }
        CardValue::Reverse => {
            game_info.swap_direction();
            game_info.advance_turn();
            game_info.advance_turn();
        }
        CardValue::DrawTwo => {
            let next_player = game_info
                .current_player
                .expect("can't play a card without a current player");
            let own_pid = socket.id().expect("server should've assigned our peer id");

            // make sure we don't draw cards for ourselves
            if next_player == *card_player {
                return;
            }

            if next_player == own_pid {
                // draw cards for main player
                let cards = deck.draw(2);
                if cards.is_empty() {
                    // no cards left in deck
                    // TODO: there should be some indicator of this
                    return;
                };
                main_player.cards.extend(&cards);

                for card in cards {
                    spawn_events.send(SpawnCard {
                        card,
                        position: CardPosition::Draw,
                        card_type: CardType::Hand,
                    });
                }
            } else {
                // increment card count for opponent
                for opponent in opponents.0.iter_mut() {
                    if opponent.id == next_player {
                        opponent.card_count += 2;
                        break;
                    }
                }
                deck.draw(2);
            }
        }
        _ => {}
    }
}

/// Handles the start/restart game event from host.
fn handle_start_game(
    mut events: EventReader<StartGame>,
    mut spawn_events: EventWriter<SpawnCard>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut menu_state: ResMut<NextState<MenuState>>,
    mut screen_state: ResMut<NextState<ScreenState>>,
    mut discard_pile: ResMut<DiscardCards>,
    mut main_player: ResMut<MainPlayer>,
    mut game_info: ResMut<GameInfo>,
    mut deck: ResMut<Deck>,
) {
    let Some(event) = events.read().next() else {
		return;
	};

    game_info.order = event.order.clone();
    game_info.current_player = event.order.first().copied();

    // construct start event packet
    let mut packet: Vec<u8> = Vec::new();
    packet.push(
        if event.restart {
            SocketEvent::Restart
        } else {
            SocketEvent::Start
        }
        .into(),
    );
    // add player order
    packet.push(event.order.len() as u8);
    for player_id in event.order.iter() {
        packet.extend_from_slice(player_id.0.as_bytes());
    }
    // add deck
    packet.extend(deck.get_card_order());
    let packet = packet.into_boxed_slice();

    // send packet to all peers
    for peer in socket.connected_peers().collect::<Vec<_>>().iter() {
        println!("sending packet: {packet:?}");
        socket.send(packet.clone(), *peer);
    }

    let own_pid = socket.id().expect("server should assign us a peer id");

    initialize_game_start(
        &own_pid,
        &mut spawn_events,
        &mut game_info,
        &mut main_player,
        &mut deck,
        &mut discard_pile,
        &mut screen_state,
        &mut menu_state,
    )
}

/// Sends draw card event to all peers and advances turn.
fn handle_draw_card(
    mut events: EventReader<DrawCard>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut game_info: ResMut<GameInfo>,
) {
    for _ in events.read() {
        let packet = Vec::from([SocketEvent::Draw.into()]).into_boxed_slice();
        for peer in socket.connected_peers().collect::<Vec<_>>().iter() {
            socket.send(packet.clone(), *peer);
        }
        game_info.advance_turn();
    }
}

/// Sends play card event to all peers and advances turn.
fn handle_play_card(
    mut play_events: EventReader<PlayCard>,
    mut win_events: EventWriter<Win>,
    mut spawn_events: EventWriter<SpawnCard>,
    mut main_player: ResMut<MainPlayer>,
    mut opponents: ResMut<Opponents>,
    mut deck: ResMut<Deck>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
    mut game_info: ResMut<GameInfo>,
) {
    for event in play_events.read() {
        let packet = Vec::from([SocketEvent::Play.into(), event.0.into()]).into_boxed_slice();
        for peer in socket.connected_peers().collect::<Vec<_>>().iter() {
            socket.send(packet.clone(), *peer);
        }
        game_info.advance_turn();

        let Some(pid) = socket.id() else { return; };
        handle_card_effect(
            &event.0,
            &pid,
            &mut spawn_events,
            &mut socket,
            &mut game_info,
            &mut main_player,
            &mut opponents,
            &mut deck,
        );

        if main_player.cards.is_empty() {
            let Some(id) = socket.id() else { return; };
            win_events.send(Win(id));
        }
    }
}

/// Handles the restart game event from host.
fn handle_restart_game(
    hand_cards: Query<Entity, With<HandCard>>,
    discard_cards: Query<Entity, With<DiscardCard>>,
    mut restart_events: EventReader<RestartGame>,
    mut start_events: EventWriter<StartGame>,
    mut game_screen_state: ResMut<NextState<GameScreenState>>,
    mut discard_pile: ResMut<DiscardCards>,
    mut game_info: ResMut<GameInfo>,
    mut main_player: ResMut<MainPlayer>,
    mut opponents: ResMut<Opponents>,
    mut commands: Commands,
) {
    if restart_events.read().next().is_none() {
        return;
    }

    // rotate player order for new game
    let mut order = game_info.order.clone();
    order.rotate_left(1);

    reset_game_state(
        &discard_cards,
        &hand_cards,
        &mut game_screen_state,
        &mut discard_pile,
        &mut main_player,
        &mut opponents,
        &mut game_info,
        &mut commands,
    );

    start_events.send(StartGame {
        order,
        restart: true,
    });
}

/// Sends wild color choice to peers.
fn handle_wild_color(
    mut wild_events: EventReader<WildColor>,
    mut socket: ResMut<MatchboxSocket<SingleChannel>>,
) {
    for event in wild_events.read() {
        let packet = Vec::from([SocketEvent::Wild.into(), event.0.into()]).into_boxed_slice();
        for peer in socket.connected_peers().collect::<Vec<_>>().iter() {
            socket.send(packet.clone(), *peer);
        }
    }
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<StartGame>()
            .add_event::<DrawCard>()
            .add_event::<PlayCard>()
            .add_event::<RestartGame>()
            .add_event::<WildColor>()
            .add_state::<ServerState>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    receive_messages,
                    handle_start_game,
                    handle_draw_card,
                    handle_play_card,
                    handle_restart_game,
                    handle_wild_color,
                )
                    .run_if(resource_exists::<MatchboxSocket<SingleChannel>>()),
            );
    }
}
