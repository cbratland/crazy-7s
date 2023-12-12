//! Main menu screens.

use crate::button::ButtonEnabled;
use crate::network::ServerState;
use crate::{despawn_screen, ScreenState};
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy_matchbox::prelude::*;

mod join;
mod lobby;
mod main;
mod settings;

/// State used for the current menu screen.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum MenuState {
    #[default]
    Disabled,
    Main,
    Join,
    Lobby,
    Settings,
}

/// Initializes the menu state to the main menu.
fn setup(mut menu_state: ResMut<NextState<MenuState>>) {
    menu_state.set(MenuState::Main);
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MenuState>()
            .add_systems(OnEnter(ScreenState::Menu), setup)
            // main menu
            .add_systems(OnEnter(MenuState::Main), main::setup)
            .add_systems(OnExit(MenuState::Main), despawn_screen::<main::OnScreen>)
            .add_systems(
                Update,
                main::handle_action.run_if(in_state(MenuState::Main)),
            )
            // join menu
            .add_systems(OnEnter(MenuState::Join), join::setup)
            .add_systems(OnExit(MenuState::Join), despawn_screen::<join::OnScreen>)
            .add_systems(
                Update,
                (
                    join::handle_action,
                    join::update_code,
                    join::update_code_display,
                    join::update_button_enabled,
                )
                    .run_if(in_state(MenuState::Join)),
            )
            // lobby menu
            // this uses onexit for serverstate none, since it should be run after
            // serverstate has been set to either server or client
            .add_systems(OnExit(ServerState::None), lobby::setup)
            .add_systems(OnExit(MenuState::Lobby), despawn_screen::<lobby::OnScreen>)
            .add_systems(
                OnEnter(ServerState::None),
                lobby::close_socket.run_if(resource_exists::<MatchboxSocket<SingleChannel>>()),
            )
            .add_systems(
                Update,
                (lobby::handle_action, lobby::update_players_text)
                    .run_if(in_state(MenuState::Lobby)),
            )
            // settings menu
            .add_systems(OnEnter(MenuState::Settings), settings::setup)
            .add_systems(
                OnExit(MenuState::Settings),
                despawn_screen::<settings::OnScreen>,
            )
            .add_systems(
                Update,
                (
                    settings::handle_action,
                    settings::update_name,
                    settings::update_name_display,
                )
                    .run_if(in_state(MenuState::Settings)),
            );
    }
}
