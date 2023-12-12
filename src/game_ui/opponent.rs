//! Opponent UI

use crate::{
    game_ui::board::OnScreen,
    info::{GameInfo, Opponents},
    ScreenState,
};
use bevy::prelude::{Plugin as BevyPlugin, *};
use bevy::sprite::MaterialMesh2dBundle;
use bevy_matchbox::matchbox_socket::PeerId;

/// Opponent highlight component, shown when it's their turn.
#[derive(Component)]
pub struct OpponentHighlight(PeerId);

// Opponent circle component (unused).
// #[derive(Component)]
// pub struct OpponentIcon(PeerId);

/// Opponent card count text component.
#[derive(Component)]
pub struct OpponentCardCount(PeerId);

/// Initializes empty opponent list.
fn setup(mut commands: Commands) {
    commands.insert_resource(Opponents(Vec::new()));
}

/// Draws circles for each opponent.
fn draw_opponents(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    opponents: Res<Opponents>,
) {
    if opponents.0.is_empty() {
        return;
    }
    let opponent_count = opponents.0.len();
    let center_idx = (opponent_count - 1) as f32 / 2.0;
    for (idx, opponent) in opponents.0.iter().enumerate() {
        let x = -160.0 * (center_idx - idx as f32);

        commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: meshes.add(shape::Circle::new(42.0).into()).into(),
                    material: materials.add(ColorMaterial::from(Color::WHITE.with_a(0.0))),
                    transform: Transform::from_translation(Vec3::new(x, 160.0, 1.0)),
                    ..default()
                },
                OpponentHighlight(opponent.id),
                OnScreen,
            ))
            .with_children(|parent| {
                // name
                parent.spawn(Text2dBundle {
                    text: Text::from_section(
                        opponent.name.clone(),
                        TextStyle {
                            font: asset_server.load("fonts/Lato-Black.ttf"),
                            font_size: 30.0,
                            color: Color::WHITE,
                        },
                    ),
                    transform: Transform::from_translation(Vec3::new(0.0, 60.0, 2.0)),
                    ..default()
                });

                parent
                    .spawn((
                        MaterialMesh2dBundle {
                            mesh: meshes.add(shape::Circle::new(35.0).into()).into(),
                            material: materials.add(ColorMaterial::from(Color::WHITE)),
                            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
                            ..default()
                        },
                        // OpponentIcon(opponent.id),
                    ))
                    .with_children(|parent| {
                        // player card count
                        parent.spawn((
                            Text2dBundle {
                                text: Text::from_section(
                                    opponent.card_count.to_string(),
                                    TextStyle {
                                        font: asset_server.load("fonts/Lato-Black.ttf"),
                                        font_size: 40.0,
                                        color: Color::BLACK,
                                    },
                                ),
                                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 2.0)),
                                ..default()
                            },
                            OpponentCardCount(opponent.id),
                        ));
                    });
            });
    }
}

/// Updates opponent card count text.
fn update_opponent_card_count(
    mut entities: Query<(&mut Text, &OpponentCardCount)>,
    opponents: Res<Opponents>,
) {
    for (mut text, entity) in entities.iter_mut() {
        let Some(opponent) = opponents.0.iter().find(|opponent| opponent.id == entity.0) else {
        	continue;
        };
        text.sections[0].value = opponent.card_count.to_string();
    }
}

/// Enables opponent highlight when it's their turn.
fn update_opponent_highlight(
    entities: Query<(&OpponentHighlight, &Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    game_info: Res<GameInfo>,
) {
    let Some(current_player) = game_info.current_player else { return; };
    for (OpponentHighlight(id), material_handle) in entities.iter() {
        if let Some(material) = materials.get_mut(material_handle.id()) {
            material.color = if current_player == *id {
                Color::WHITE.with_a(0.15)
            } else {
                Color::WHITE.with_a(0.0)
            };
        }
    }
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(OnEnter(ScreenState::Game), draw_opponents)
            .add_systems(
                Update,
                (update_opponent_card_count, update_opponent_highlight)
                    .run_if(in_state(ScreenState::Game)),
            );
    }
}
