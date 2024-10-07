use std::time::Duration;

use bevy::{
    prelude::*,
    transform::commands,
    window::{EnabledButtons, PresentMode, PrimaryWindow, WindowResolution},
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};
use rand::Rng;

#[derive(Component)]
struct Player(f32, i32);

#[derive(Component)]
struct Apple;

#[derive(Resource)]
struct AppleSpawner {
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
            (player_movement, apple_movement, apple_spawning).run_if(in_state(GameState::Loaded)),
        )
        // .add_systems(Update, test)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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
                    translation: Vec3::new(0., -window.height() / 2. + texture_size.y / 2., 0.0),
                    ..default()
                },
                texture: texture_handle,
                ..default()
            })
            .insert(Player(300., 0));
    }
    next_state.set(GameState::Loaded);
    commands.insert_resource(AppleSpawner {
        timer: Timer::new(Duration::from_secs_f32(1.75), TimerMode::Repeating),
    });
}

fn player_movement(
    mut player_query: Query<(&mut Transform, &Player, &Handle<Image>), With<Player>>,
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    assets: Res<Assets<Image>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    let (mut transform, player, texture_handle) = player_query.single_mut();
    let texture = match assets.get(texture_handle) {
        Some(tex) => tex,
        None => return,
    };
    let texture_size = texture.size_f32();
    let window = match windows.get_single() {
        Ok(win) => win,
        Err(_) => return,
    };

    let movement = player.0 * time.delta_seconds();

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
    mut apple_query: Query<(&mut Transform, &Apple, Entity), With<Apple>>,
    time: Res<Time>,
    assets: Res<Assets<Image>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    image_assets: Res<ImageAssets>,
    mut commands: Commands,
) {
    let window = match windows.get_single() {
        Ok(win) => win,
        Err(_) => return,
    };
    let texture = match assets.get(&image_assets.apple.clone()) {
        Some(tex) => tex,
        None => return,
    };
    let texture_size = texture.size_f32();
    let bottom = -window.height() / 2. - texture_size.y / 4.;
    for (mut transform, apple, entity) in apple_query.iter_mut() {
        transform.translation.y -= 80. * time.delta_seconds();

        if transform.translation.y < bottom {
            commands.entity(entity).despawn();
        }
    }
}

fn apple_spawning(
    mut commands: Commands,
    time: Res<Time>,
    image_assets: Res<ImageAssets>,
    mut spawner: ResMut<AppleSpawner>,
    windows: Query<&Window, With<PrimaryWindow>>,
    assets: Res<Assets<Image>>,
) {
    let window = match windows.get_single() {
        Ok(win) => win,
        Err(_) => return,
    };
    spawner.timer.tick(time.delta());

    let texture = match assets.get(&image_assets.apple.clone()) {
        Some(tex) => tex,
        None => return,
    };
    let texture_size = texture.size_f32();
    if spawner.timer.finished() {
        let mut rng = rand::thread_rng();
        let spawn_range = (window.width() - (texture_size.x) / 2.) / 2.;

        let spawn_x = rng.gen_range(-spawn_range..spawn_range);

        commands
            .spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(spawn_x, 0., 0.),
                    scale: Vec3::splat(0.5),
                    ..default()
                },
                texture: image_assets.apple.clone(),
                ..default()
            })
            .insert(Apple);
    }
}
