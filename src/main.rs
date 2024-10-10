use std::time::Duration;

use bevy::{
    math::bounding::{Aabb2d, IntersectsVolume},
    prelude::*,
    window::{EnabledButtons, PresentMode, PrimaryWindow},
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};
use rand::Rng;

const PLAYER_MOVEMENT_SPEED: f32 = 300.;
const APPLE_MOVEMENT_SPEED: f32 = 150.;

#[derive(Component)]
struct Player;

#[derive(Component)]
struct Apple;

#[derive(Component)]
struct SpriteSize(Vec2);

#[derive(Component)]
struct PointsText;

#[derive(Resource)]
struct Scoreboard {
    score: i32,
}

#[derive(Resource)]
struct AppleSpawnerConfig {
    timer: Timer,
}

#[derive(AssetCollection, Resource)]
struct ImageAssets {
    #[asset(path = "textures/basket.png")]
    pub player: Handle<Image>,
    #[asset(path = "textures/apple.png")]
    pub apple: Handle<Image>,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum GameState {
    #[default]
    Loading,
    Next,
    Loaded,
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Applecatcher".to_string(),
                        resizable: false,
                        present_mode: PresentMode::AutoNoVsync,
                        enabled_buttons: EnabledButtons {
                            maximize: false,
                            ..default()
                        },
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::Next)
                .load_collection::<ImageAssets>(),
        )
        .add_systems(Startup, setup)
        .add_systems(Update, use_asset_handles.run_if(in_state(GameState::Next)))
        .add_systems(
            Update,
            (
                apple_catching,
                player_movement,
                apple_movement,
                apple_spawning,
            )
                .run_if(in_state(GameState::Loaded)),
        )
        // .add_systems(Update, test)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
    commands.insert_resource(Scoreboard { score: 0 });
    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Points: ",
                TextStyle {
                    font_size: 30.,
                    ..default()
                },
            ),
            TextSection::new(
                "0",
                TextStyle {
                    font_size: 30.,
                    ..default()
                },
            ),
        ]),
        PointsText,
    ));
}

fn use_asset_handles(
    mut commands: Commands,
    image_assets: Res<ImageAssets>,
    assets: Res<Assets<Image>>,
    mut next_state: ResMut<NextState<GameState>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let window = windows.single();
    {
        let texture_handle = image_assets.player.clone();
        let texture = assets.get(&texture_handle).unwrap();
        let texture_size = texture.size_f32();
        commands
            .spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0., -window.height() / 2. + texture_size.y / 2., 1.0),
                    ..default()
                },
                texture: texture_handle,
                ..default()
            })
            .insert(Player)
            .insert(SpriteSize(texture_size));
    }
    next_state.set(GameState::Loaded);
    commands.insert_resource(AppleSpawnerConfig {
        timer: Timer::new(Duration::from_secs_f32(1.75), TimerMode::Repeating),
    });
}

fn player_movement(
    mut player_query: Query<(&mut Transform, &SpriteSize), With<Player>>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let (mut transform, size) = player_query.single_mut();
    let texture_size = size.0;
    let window = match windows.get_single() {
        Ok(win) => win,
        Err(_) => return,
    };

    let movement = PLAYER_MOVEMENT_SPEED * time.delta_seconds();

    if keyboard_input.pressed(KeyCode::KeyA) {
        transform.translation.x -= movement;
    } else if keyboard_input.pressed(KeyCode::KeyD) {
        transform.translation.x += movement;
    }

    let left_side = -window.width() / 2. + texture_size.x / 2.;
    let ride_side = window.width() / 2. - texture_size.x / 2.;
    if transform.translation.x < left_side {
        transform.translation.x = left_side;
    } else if transform.translation.x > ride_side {
        transform.translation.x = ride_side;
    }
}

fn apple_movement(
    mut apple_query: Query<(&mut Transform, &SpriteSize, Entity), With<Apple>>,
    time: Res<Time>,
    windows: Query<&Window, With<PrimaryWindow>>,
    mut commands: Commands,
) {
    let window = match windows.get_single() {
        Ok(win) => win,
        Err(_) => return,
    };
    for (mut transform, size, entity) in apple_query.iter_mut() {
        transform.translation.y -= APPLE_MOVEMENT_SPEED * time.delta_seconds();
        let bottom = -window.height() / 2. - (size.0.y * transform.scale.y) / 2.;

        if transform.translation.y < bottom {
            commands.entity(entity).despawn();
        }
    }
}

fn apple_spawning(
    mut commands: Commands,
    time: Res<Time>,
    image_assets: Res<ImageAssets>,
    mut spawner: ResMut<AppleSpawnerConfig>,
    windows: Query<&Window, With<PrimaryWindow>>,
    assets: Res<Assets<Image>>,
) {
    spawner.timer.tick(time.delta());
    if spawner.timer.finished() {
        let window = match windows.get_single() {
            Ok(win) => win,
            Err(_) => return,
        };

        let texture = match assets.get(&image_assets.apple.clone()) {
            Some(tex) => tex,
            None => return,
        };
        let texture_size = texture.size_f32();
        let top = window.height() / 2. + texture_size.y / 4.;

        let mut rng = rand::thread_rng();
        let spawn_range = (window.width() - (texture_size.x) / 2.) / 2.;

        let spawn_x = rng.gen_range(-spawn_range..=spawn_range);

        commands
            .spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(spawn_x, top, 0.),
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                texture: image_assets.apple.clone(),
                ..default()
            })
            .insert(Apple)
            .insert(SpriteSize(texture_size));
    }
}

fn apple_catching(
    mut commands: Commands,
    apple_query: Query<(&Transform, &SpriteSize, Entity), With<Apple>>,
    player_query: Query<(&Transform, &SpriteSize), With<Player>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut points_text_query: Query<&mut Text, With<PointsText>>,
) {
    let (player_transform, player_size) = player_query.single();
    let mut points_text = points_text_query.single_mut();

    let player_aabb = Aabb2d::new(
        player_transform.translation.truncate(),
        (player_size.0 * player_transform.scale.truncate()) / 2.,
    );

    for (transform, size, entity) in apple_query.iter() {
        let box_aabb = Aabb2d::new(
            transform.translation.truncate(),
            (size.0 * transform.scale.truncate()) / 2.,
        );
        if player_aabb.intersects(&box_aabb) {
            scoreboard.score += 1;
            points_text.sections[1].value = scoreboard.score.to_string();
            // println!("Your score is now: {}", scoreboard.score);
            commands.get_entity(entity).unwrap().despawn();
        }
    }
}
