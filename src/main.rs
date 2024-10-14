use bevy::{
    prelude::*,
    window::{EnabledButtons, PresentMode},
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{config::ConfigureLoadingState, LoadingState, LoadingStateAppExt},
};

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
    MainMenu,
    Game,
}

#[derive(Clone, Eq, PartialEq, Debug, Hash, Default, States)]
enum PauseMode {
    #[default]
    Playing,
    Paused,
}

#[derive(Resource)]
struct Scoreboard {
    score: i32,
}

#[derive(Resource)]
struct AppleSpawnerConfig {
    timer: Timer,
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

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
        .init_state::<PauseMode>()
        .add_loading_state(
            LoadingState::new(GameState::Loading)
                .continue_to_state(GameState::MainMenu)
                .load_collection::<ImageAssets>(),
        )
        .add_systems(Startup, setup)
        .add_plugins(({ main_menu::main_menu_plugin }, { game::game_plugin }, {
            pause_menu::pause_menu_plugin
        }))
        // .add_systems(Update, test)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

mod main_menu {
    use std::time::Duration;

    use bevy::prelude::*;

    use super::{
        despawn_screen, AppleSpawnerConfig, GameState, Scoreboard, HOVERED_BUTTON, NORMAL_BUTTON,
        PRESSED_BUTTON,
    };

    #[derive(Component)]
    struct OnMainMenuScreen;

    #[derive(Component)]
    enum MenuButtonAction {
        Play,
        Quit,
    }

    pub fn main_menu_plugin(app: &mut App) {
        app.add_systems(OnEnter(GameState::MainMenu), setup)
            .add_systems(
                Update,
                (button_system, menu_action).run_if(in_state(GameState::MainMenu)),
            )
            .add_systems(
                OnExit(GameState::MainMenu),
                despawn_screen::<OnMainMenuScreen>,
            );
    }

    fn setup(mut commands: Commands) {
        let button_style = Style {
            width: Val::Px(250.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnMainMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Play,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "New Game",
                                    button_text_style.clone(),
                                ));
                            });

                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Quit,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Quit",
                                    button_text_style.clone(),
                                ));
                            });
                    });
            });
    }

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color) in &mut interaction_query {
            *color = match *interaction {
                Interaction::Pressed => PRESSED_BUTTON,
                Interaction::Hovered => HOVERED_BUTTON,
                Interaction::None => NORMAL_BUTTON,
            }
            .into();
        }
    }

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_events: EventWriter<AppExit>,
        mut game_state: ResMut<NextState<GameState>>,
        mut commands: Commands,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Play => {
                        commands.insert_resource(Scoreboard { score: 0 });
                        commands.insert_resource(AppleSpawnerConfig {
                            timer: Timer::new(Duration::from_secs_f32(1.75), TimerMode::Repeating),
                        });
                        game_state.set(GameState::Game);
                    }
                    MenuButtonAction::Quit => {
                        app_exit_events.send(AppExit::Success);
                    }
                }
            }
        }
    }
}

mod game {
    use bevy::{
        math::bounding::{Aabb2d, IntersectsVolume},
        prelude::*,
        window::PrimaryWindow,
    };

    use crate::PauseMode;

    use super::{despawn_screen, AppleSpawnerConfig, GameState, ImageAssets, Scoreboard};

    use rand::Rng;

    #[derive(Component)]
    struct OnGameScreen;

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

    pub fn game_plugin(app: &mut App) {
        app.add_systems(OnEnter(GameState::Game), setup)
            .add_systems(
                Update,
                (
                    apple_catching,
                    player_movement,
                    apple_movement,
                    apple_spawning,
                )
                    .run_if(in_state(GameState::Game).and_then(in_state(PauseMode::Playing))),
            )
            .add_systems(OnExit(GameState::Game), despawn_screen::<OnGameScreen>);
    }

    fn setup(
        mut commands: Commands,
        image_assets: Res<ImageAssets>,
        assets: Res<Assets<Image>>,
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
                        translation: Vec3::new(
                            0.,
                            -window.height() / 2. + texture_size.y / 2.,
                            1.0,
                        ),
                        ..default()
                    },
                    texture: texture_handle,
                    ..default()
                })
                .insert(Player)
                .insert(SpriteSize(texture_size))
                .insert(OnGameScreen);
        }
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
            OnGameScreen,
        ));
    }

    fn player_movement(
        mut player_query: Query<(&mut Transform, &SpriteSize), With<Player>>,
        time: Res<Time>,
        keyboard_input: Res<ButtonInput<KeyCode>>,
        windows: Query<&Window, With<PrimaryWindow>>,
        mut game_state: ResMut<NextState<PauseMode>>,
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

        if keyboard_input.just_pressed(KeyCode::Escape) {
            game_state.set(PauseMode::Paused);
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
                .insert(OnGameScreen)
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
}

mod pause_menu {
    use bevy::prelude::*;

    use crate::{despawn_screen, PauseMode, HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON};

    #[derive(Component)]
    struct OnPauseMenuScreen;

    #[derive(Component)]
    enum MenuButtonAction {
        Resume,
        Quit,
    }

    pub fn pause_menu_plugin(app: &mut App) {
        app.add_systems(OnEnter(PauseMode::Paused), setup)
            .add_systems(
                Update,
                (button_system, menu_action, keyboard_input).run_if(in_state(PauseMode::Paused)),
            )
            .add_systems(
                OnExit(PauseMode::Paused),
                despawn_screen::<OnPauseMenuScreen>,
            );
    }

    fn setup(mut commands: Commands) {
        let button_style = Style {
            width: Val::Px(250.0),
            height: Val::Px(65.0),
            margin: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let button_text_style = TextStyle {
            font_size: 40.0,
            ..default()
        };

        commands
            .spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    ..default()
                },
                OnPauseMenuScreen,
            ))
            .with_children(|parent| {
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        ..default()
                    })
                    .with_children(|parent| {
                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Resume,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Resume",
                                    button_text_style.clone(),
                                ));
                            });

                        parent
                            .spawn((
                                ButtonBundle {
                                    style: button_style.clone(),
                                    background_color: NORMAL_BUTTON.into(),
                                    ..default()
                                },
                                MenuButtonAction::Quit,
                            ))
                            .with_children(|parent| {
                                parent.spawn(TextBundle::from_section(
                                    "Quit",
                                    button_text_style.clone(),
                                ));
                            });
                    });
            });
    }

    fn keyboard_input(
        keyboard_input: Res<ButtonInput<KeyCode>>,
        mut game_state: ResMut<NextState<PauseMode>>,
    ) {
        if keyboard_input.just_pressed(KeyCode::Escape){
            game_state.set(PauseMode::Playing);
        }
    }

    fn button_system(
        mut interaction_query: Query<
            (&Interaction, &mut BackgroundColor),
            (Changed<Interaction>, With<Button>),
        >,
    ) {
        for (interaction, mut color) in &mut interaction_query {
            *color = match *interaction {
                Interaction::Pressed => PRESSED_BUTTON,
                Interaction::Hovered => HOVERED_BUTTON,
                Interaction::None => NORMAL_BUTTON,
            }
            .into();
        }
    }

    fn menu_action(
        interaction_query: Query<
            (&Interaction, &MenuButtonAction),
            (Changed<Interaction>, With<Button>),
        >,
        mut app_exit_events: EventWriter<AppExit>,
        mut game_state: ResMut<NextState<PauseMode>>,
    ) {
        for (interaction, menu_button_action) in &interaction_query {
            if *interaction == Interaction::Pressed {
                match menu_button_action {
                    MenuButtonAction::Resume => {
                        game_state.set(PauseMode::Playing);
                    }
                    MenuButtonAction::Quit => {
                        app_exit_events.send(AppExit::Success);
                    }
                }
            }
        }
    }
}

fn despawn_screen<T: Component>(to_despawn: Query<Entity, With<T>>, mut commands: Commands) {
    for entity in &to_despawn {
        commands.entity(entity).despawn_recursive();
    }
}
