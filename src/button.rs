//! Button handling.

use crate::card::CARD_ANIMATION_SPEED;
use bevy::prelude::{Plugin as BevyPlugin, *};

/// Indicates whether a button is enabled or not.
///
/// If a button is disabled, it won't respond to interactions.
#[derive(Component)]
pub struct ButtonEnabled(pub bool);

/// Indicates a button is being held down.
#[derive(Component)]
pub struct Pressed;

/// Indicates a button is being hovered over.
#[derive(Component)]
pub struct Hovered;

/// Determines if buttons are being hovered over or pressed.
fn button_system(
    mut interaction_query: Query<
        (
            Entity,
            &Interaction,
            Option<&ButtonEnabled>,
            &mut BackgroundColor,
        ),
        (
            Or<(Changed<Interaction>, Changed<ButtonEnabled>)>,
            With<Button>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, interaction, enabled, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                commands.entity(entity).insert(Pressed);
                commands.entity(entity).remove::<Hovered>();
            }
            Interaction::Hovered => {
                commands.entity(entity).insert(Hovered);
                commands.entity(entity).remove::<Pressed>();
            }
            Interaction::None => {
                commands.entity(entity).remove::<Hovered>();
                commands.entity(entity).remove::<Pressed>();
            }
        }
        *color = if enabled.map_or(true, |e| e.0) {
            // match *interaction {
            //     Interaction::Pressed => Color::GRAY.into(),
            //     Interaction::Hovered => Color::GREEN.into(),
            //     Interaction::None => Color::WHITE.into(),
            // }
            Color::WHITE.into()
        } else {
            Color::rgba(1.0, 1.0, 1.0, 0.3).into()
        };
    }
}

/// Resizes button to the normal size.
fn animate_button_default(
    mut buttons: Query<&mut Transform, (With<Button>, Without<Hovered>, Without<Pressed>)>,
    time: Res<Time>,
) {
    let card_speed = CARD_ANIMATION_SPEED * time.delta_seconds();
    let target = 1.0;

    for mut transform in &mut buttons {
        let current = transform.scale.x;
        let distance = target - current;
        if distance.abs() < 0.01 {
            continue;
        }
        transform.scale.x += distance * card_speed;
        transform.scale.y += distance * card_speed;
    }
}

/// Scales up buttons that are being hovered over.
fn animate_button_hover(
    // hand: Query<&GlobalTransform, With<PlayerHand>>,
    mut buttons: Query<&mut Transform, (With<Button>, With<Hovered>)>,
    time: Res<Time>,
) {
    let card_speed = CARD_ANIMATION_SPEED * time.delta_seconds();
    let target = 1.05;

    for mut transform in &mut buttons {
        let current = transform.scale.x;
        let distance = target - current;
        if distance.abs() < 0.01 {
            continue;
        }
        transform.scale.x += distance * card_speed;
        transform.scale.y += distance * card_speed;
    }
}

/// Scales down buttons that are being pressed.
fn animate_button_press(
    mut buttons: Query<&mut Transform, (With<Button>, With<Pressed>)>,
    time: Res<Time>,
) {
    let card_speed = CARD_ANIMATION_SPEED * time.delta_seconds();
    let target = 0.95;

    for mut transform in &mut buttons {
        let current = transform.scale.x;
        let distance = target - current;
        if distance.abs() < 0.01 {
            continue;
        }
        transform.scale.x += distance * card_speed;
        transform.scale.y += distance * card_speed;
    }
}

pub struct Plugin;

impl BevyPlugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                button_system,
                animate_button_default,
                animate_button_hover,
                animate_button_press,
            ),
        );
    }
}
