use bevy::prelude::*;

#[derive(Component)]
struct Player;



fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .insert_resource(ClearColor(Color::rgb(0.3, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        Player,
        SpriteBundle {
            texture: asset_server.load("spaceship.png"),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .with_scale(Vec3::splat(2.0)),
            ..default()
        }
    ));
}
