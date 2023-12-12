//! A multiplayer uno-like card game made with Bevy and matchbox.

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    window::{PresentMode, PrimaryWindow},
    winit::WinitSettings,
};
use rand::Rng;

pub const SERVER_URL: &str = "ws://127.0.0.1:3536";

const SCREEN_WIDTH_DEFAULT: f32 = 800.0;
const SCREEN_HEIGHT_DEFAULT: f32 = 500.0;
const SCREEN_MAX_SCALE: f32 = 2.0; // needs to also be used in background.wgsl

mod button;
mod card;
mod deck;
mod game_ui;
mod info;
mod menu;
mod network;
mod screens;
mod storage;

/// The global screen state.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum ScreenState {
    // Splash,
    #[default]
    Menu,
    Game,
}

/// The screen state for the game screen.
#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameScreenState {
    #[default]
    Game,
    WildColor,
    Win,
}

/// Tiled background shader material.
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BackgroundMaterial {
    #[texture(0)]
    #[sampler(1)]
    image: Option<Handle<Image>>,
}

impl Material2d for BackgroundMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/background.wgsl".into()
    }
}

/// Component for the main camera.
#[derive(Component)]
pub struct MainCamera;

/// Coordinates of the mouse cursor in world space.
#[derive(Resource, Default)]
struct WorldCoords(Vec2);

/// The username of the player.
///
/// This is loaded from storage, or generated if it doesn't exist.
#[derive(Resource)]
pub struct Username(String);

/// Draws background and sets up camera and storage.
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<BackgroundMaterial>>,
    mut framepace_settings: ResMut<bevy_framepace::FramepaceSettings>,
    asset_server: Res<AssetServer>,
) {
    framepace_settings.limiter = bevy_framepace::Limiter::from_framerate(120.0);

    let mut storage = storage::Storage::new();

    let username = if let Ok(username) = storage.get("username") {
        username
    } else {
        let user_num = rand::thread_rng().gen_range(1000..10000);
        let username = format!("User {user_num}");
        if let Err(err) = storage.set("username", &username.clone()) {
            println!("Error saving username: {:?}", err);
        }
        username
    };

    commands.insert_resource(Username(username));
    commands.insert_resource(storage);
    commands.init_resource::<WorldCoords>();

    // draw background
    commands.spawn(MaterialMesh2dBundle {
        // mesh: meshes.add(shape::Plane { size: 3.0 }.into()).into(),
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default().with_scale(Vec3::new(
            SCREEN_WIDTH_DEFAULT * SCREEN_MAX_SCALE,
            SCREEN_HEIGHT_DEFAULT * SCREEN_MAX_SCALE,
            0.0,
        )),
        material: materials.add(BackgroundMaterial {
            image: Some(asset_server.load("textures/background.png")),
        }),
        ..default()
    });

    commands.spawn((Camera2dBundle::default(), MainCamera));
}

/// Tracks the mouse cursor position in world space.
fn handle_cursor(
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut coords: ResMut<WorldCoords>,
) {
    let (camera, camera_transform) = camera.single();
    let window = window.single();

    // convert cursor position into world coordinates and truncate to get rid of z
    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        coords.0 = world_position;
    }
}

/// Generic system that takes a component as a parameter, and will despawn all entities with that component
fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "crazy 7s".into(),
                        resolution: (800., 500.).into(),
                        resize_constraints: WindowResizeConstraints {
                            min_width: SCREEN_WIDTH_DEFAULT,
                            max_width: SCREEN_WIDTH_DEFAULT * SCREEN_MAX_SCALE,
                            min_height: SCREEN_HEIGHT_DEFAULT,
                            max_height: SCREEN_HEIGHT_DEFAULT * SCREEN_MAX_SCALE,
                        },
                        present_mode: PresentMode::AutoVsync,
                        // Tells wasm to resize the window according to the available canvas
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            Material2dPlugin::<BackgroundMaterial>::default(),
            bevy_framepace::FramepacePlugin,
        ))
        // .add_plugins((
        //     bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        //     bevy::diagnostic::LogDiagnosticsPlugin::default(),
        // ))
        .insert_resource(WinitSettings::game())
        .add_state::<ScreenState>()
        .add_state::<GameScreenState>()
        .add_systems(Startup, setup)
        .add_systems(Update, handle_cursor)
        .add_plugins((
            menu::Plugin,
            info::Plugin,
            card::Plugin,
            deck::Plugin,
            network::Plugin,
            button::Plugin,
            game_ui::board::Plugin,
            game_ui::hand::Plugin,
            game_ui::opponent::Plugin,
            screens::win::Plugin,
            screens::wild::Plugin,
        ))
        .run();
}
