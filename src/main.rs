use bevy::prelude::*;

#[derive(Component)]
struct Basket;

#[derive(Component)]
struct Apple;

fn main() {
    App::new()
    .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
    .add_systems(Startup, spawn_camera)
    .add_systems(Startup, spawn_sprite)
    .run();
}

fn spawn_camera(mut commands:Commands){
    commands.spawn(Camera2d::default());
}

fn spawn_sprite(mut commands:Commands, asset_server:Res<AssetServer>){
    let texture = asset_server.load("textures/apple.png");
    commands.spawn(SpriteBundle{
        transform: Transform::from_scale(Vec3::splat(6.0)),
        texture,
        ..default()
    });
}