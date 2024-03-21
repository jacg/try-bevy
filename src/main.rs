use std::time::Duration;

use bevy::prelude::*;
use rand::Rng;

#[derive(Component)]
struct Player;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites
        ))
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_systems(Startup, setup)
        .insert_resource(AsteroidSpawnTimer(Timer::from_seconds(
            1.0,
            TimerMode::Once,
        )))
        .add_systems(
            Update,
            (
                sprite_movement,
                ship_movement_input,
                confine_player_to_screen,
                bullet_firing,
                spawn_asteroids,
                despawn_entities_outside_of_screen,
                asteroid_bullet_collision,
            ),
        )
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    // Spawn the spaceship
    commands.spawn((
        Player,
        SpriteBundle {
            texture: asset_server.load("spaceship.png"),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.0))
                .with_scale(Vec3::splat(2.)),
            ..default()
        },
        SpriteMovement { vx: 0.0, vy: 0.0 },
        CooldownTimer(Timer::from_seconds(0.2, TimerMode::Once)),
        BallCollider(18.0),
    ));
}

#[derive(Component)]
struct SpriteMovement { vx: f32, vy: f32 }

fn sprite_movement(time: Res<Time>, mut sprite_position: Query<(&SpriteMovement, &mut Transform)>) {
    for (&SpriteMovement { vx, vy }, mut transform) in &mut sprite_position {
        let dt = time.delta_seconds();
        transform.translation += Vec3 { x: vx, y: vy, z: 0.0} * dt;
    }
}

fn ship_movement_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut player: Query<&mut SpriteMovement, With<Player>>,
) {
    let mut sprite_movement = player.single_mut();

    macro_rules! kbd {
        ($direction:ident $sign:literal $k1:ident $k2:ident) => {
            if keyboard_input.pressed(KeyCode::$k1) ||
               keyboard_input.pressed(KeyCode::$k2) {
                sprite_movement.$direction += $sign * 1.0;
            }
        };
    }

    kbd!(vx -1.0 Comma  Left);
    kbd!(vx  1.0 P      Right);
    kbd!(vy  1.0 Key3   Up);
    kbd!(vy -1.0 Period Down);
}

fn confine_player_to_screen(
    mut player: Query<(&mut Transform, &mut SpriteMovement), With<Player>>,
    window: Query<&Window>,
) {
    let window = window.single();
    let half_width = window.resolution.width() / 2.0;
    let half_height = window.resolution.height() / 2.0;

    let (mut transform, mut movement) = player.single_mut();

    let t = &mut transform.translation;
    if t.x.abs() > half_width  && t.x.signum() == movement.vx.signum() { movement.vx = 0.0; (*t).x = half_width  * t.x.signum(); }
    if t.y.abs() > half_height && t.y.signum() == movement.vy.signum() { movement.vy = 0.0; (*t).y = half_height * t.y.signum(); }

}

#[derive(Component)]
struct Bullet;

#[derive(Component, Deref, DerefMut)]
struct CooldownTimer(Timer);

fn bullet_firing(
    mut cmd: Commands,
    mut player: Query<(&Transform, &mut CooldownTimer), With<Player>>,
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    asset_server: Res<AssetServer>,
) {
    let (translation, mut timer) = player.single_mut();
    timer.tick(time.delta());
    let position = translation.translation + Vec3::new(0.0, 30.0, 0.0);

    if keyboard_input.pressed(KeyCode::Space) && timer.finished() {
        cmd.spawn((
            Bullet,
            SpriteBundle {
                texture: asset_server.load("bullet.png"),
                transform: Transform::from_translation(position),
                ..default()
            },
            SpriteMovement { vx: 0.0, vy: 200.0 },
            BallCollider(2.0),
        ));
        timer.reset();
    }
}

#[derive(Component)]
struct Asteroid;

#[derive(Resource, Deref, DerefMut)]
struct AsteroidSpawnTimer(Timer);

fn spawn_asteroids(
    mut cmd: Commands,
    window: Query<&Window>,
    time: Res<Time>,
    mut timer: ResMut<AsteroidSpawnTimer>,
    asset_server: Res<AssetServer>,
) {
    timer.tick(time.delta());

    if timer.finished() {
        let mut rng = rand::thread_rng();

        let window = window.single();
        let half_width = window.resolution.width() / 2.0;
        let half_height = window.resolution.height() / 2.0;

        // Spawn an asteroid
        cmd.spawn((
            Asteroid,
            SpriteBundle {
                texture: asset_server.load("asteroid.png"),
                transform: Transform::from_translation(Vec3::new(
                    rng.gen_range(-half_width..half_width),
                    half_height + 100.0, // Add buffer so that objects don't appear on screen from thin air
                    0.0,
                ))
                .with_scale(Vec3::splat(2.0)),
                ..default()
            },
            SpriteMovement { vx: 0.0, vy: -30.0 },
            BallCollider(24.0),
        ));
        timer.set_duration(Duration::from_millis(rng.gen_range(500..3000)));
        timer.reset();
    }
}

fn despawn_entities_outside_of_screen(
    mut cmd: Commands,
    window: Query<&Window>,
    query: Query<(Entity, &Transform), Or<(With<Asteroid>, With<Bullet>)>>,
) {
    let window = window.single();
    // Add buffer so that objects aren't despawned until they are completely off the screen
    let half_height = window.resolution.height() / 2.0 + 100.0;

    for (entity, transform) in &mut query.iter() {
        if transform.translation.y < -half_height || transform.translation.y > half_height {
            cmd.entity(entity).despawn();
        }
    }
}

#[derive(Component)]
struct BallCollider(f32);

fn asteroid_bullet_collision(
    mut commands: Commands,
    bullets  : Query<(Entity, &Transform, &BallCollider), With<Bullet>>,
    asteroids: Query<(Entity, &Transform, &BallCollider), With<Asteroid>>,
) {
    for (bullet_entity, bullet_transform, bullet_collider) in &mut bullets.iter() {
        for (asteroid_entity, asteroid_transform, asteroid_collider) in &mut asteroids.iter() {
            if bullet_transform
                .translation
                .distance(asteroid_transform.translation)
                < bullet_collider.0 + asteroid_collider.0
            {
                commands.entity(bullet_entity).despawn();
                commands.entity(asteroid_entity).despawn();
            }
        }
    }
}
