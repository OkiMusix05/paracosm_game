use std::time::Duration;
use bevy::prelude::*;
use bevy::render::camera::ScalingMode;
use crate::thing::ThingPlugin;

mod thing;

fn main () {
    App::new()
        .add_plugins(DefaultPlugins
            .set(ImagePlugin::default_nearest())
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Paracosm".into(),
                    resolution: (1280.0, 720.0).into(),
                    resizable: false,
                    ..default()
                }),
                ..default()
            })//.build(),
        )
        .insert_resource(Money(100.))
        .add_plugins(ThingPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (animate_sprite, execute_animations))
        .add_systems(Update, (character_movement))
        .run();
}
#[derive(Component)]
pub struct Player {
    pub speed: f32,
    pub state: PlayerState,
    pub last_direction: Option<Direction>,
    pub idle_animations: DirectionalAnimations,
}
#[derive(PartialEq, Eq)]
enum PlayerState {
    Idle,
    Moving,
}
#[derive(Clone)]
pub struct DirectionalAnimations {
    pub up: AnimationIndices,
    pub down: AnimationIndices,
    pub left: AnimationIndices,
    pub right: AnimationIndices,
}
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    RU, // Right Up
    LU, // Left Up
    RD, // Right Down
    LD, // Left Down
}
fn character_movement(
    mut characters: Query<(&mut Transform, &mut Player)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, mut player) in &mut characters {
        let mut movement = Vec2::ZERO;
        if input.pressed(KeyCode::KeyE) { movement.y += 1.0; movement.x += 1.0; player.last_direction = Some(Direction::RU); }
        if input.pressed(KeyCode::KeyQ) { movement.y += 1.0; movement.x -= 1.0; player.last_direction = Some(Direction::LU);}
        if input.pressed(KeyCode::KeyD) { movement.y -= 1.0; movement.x += 1.0; player.last_direction = Some(Direction::RD);}
        if input.pressed(KeyCode::KeyA) { movement.y -= 1.0; movement.x -= 1.0; player.last_direction = Some(Direction::LD);}
        if movement.length_squared() > 0.0 {
            movement = movement.normalize();
            player.state = PlayerState::Moving;
        } else {
            player.state = PlayerState::Idle;
        }
        let movement_amount = movement * player.speed * time.delta_seconds();
        transform.translation += movement_amount.extend(0.0);
    }
}

#[derive(Resource)]
pub struct Money(pub f32);

fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>) {
    let mut camera = Camera2dBundle::default();

    camera.projection.scaling_mode = ScalingMode::AutoMin {
        min_width: 1280.,
        min_height: 720.
    };
    commands.spawn(camera);

    /*let texture = asset_server.load("Characters/Base/Unarmed_Idle_full.png");
    commands.spawn((
        SpriteBundle {
            texture,
            transform: Transform::IDENTITY.with_scale(Vec3::splat(2.)),
            ..default()
        },
        Player { speed: 240.0, state: PlayerState::Idle },
    ));*/
    let idle_texture_atlas = asset_server.load("Characters/Base/Unarmed_Idle_full.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(64), 12, 4, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let animation_indices = AnimationIndices { first: 0, last: 4 };
    let idle_animations = DirectionalAnimations {
        up: AnimationIndices { first: 36, last: 36 },
        down: AnimationIndices { first: 0, last: 4 },
        left: AnimationIndices { first: 12, last: 15 },
        right: AnimationIndices { first: 6, last: 10},
    };
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_scale(Vec3::splat(3.0)),
            texture: idle_texture_atlas.clone(),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout.clone(),
            index: animation_indices.first,
        },
        Player {speed: 240.0, last_direction: None, state: PlayerState::Idle, idle_animations,},
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating))
    ));
}


// Animation
// This system runs when the user clicks the left arrow key or right arrow key
fn trigger_animation<S: Component>(mut query: Query<&mut AnimationConfig, With<S>>) {
    // we expect the Component of type S to be used as a marker Component by only a single entity
    let mut animation = query.single_mut();
    // we create a new timer when the animation is triggered
    animation.frame_timer = AnimationConfig::timer_from_fps(animation.fps);
}

#[derive(Component)]
struct AnimationConfig {
    first_sprite_index: usize,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
}

impl AnimationConfig {
    fn new(first: usize, last: usize, fps: u8) -> Self {
        Self {
            first_sprite_index: first,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps),
        }
    }

    fn timer_from_fps(fps: u8) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), TimerMode::Once)
    }
}
#[derive(Component, Clone)]
struct AnimationIndices {
    first: usize,
    last: usize,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

// This system loops through all the sprites in the `TextureAtlas`, from  `first_sprite_index` to
// `last_sprite_index` (both defined in `AnimationConfig`).
fn execute_animations(
    time: Res<Time>,
    mut query: Query<(&mut AnimationConfig, &mut TextureAtlas)>,
) {
    for (mut config, mut atlas) in &mut query {
        // we track how long the current sprite has been displayed for
        config.frame_timer.tick(time.delta());

        // If it has been displayed for the user-defined amount of time (fps)...
        if config.frame_timer.just_finished() {
            if atlas.index == config.last_sprite_index {
                // ...and it IS the last frame, then we move back to the first frame and stop.
                atlas.index = config.first_sprite_index;
            } else {
                // ...and it is NOT the last frame, then we move to the next frame...
                atlas.index += 1;
                // ...and reset the frame timer to start counting all over again
                config.frame_timer = AnimationConfig::timer_from_fps(config.fps);
            }
        }
    }
}
fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&Player, &mut AnimationTimer, &mut TextureAtlas)>,
) {
    for (player, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());

        // Use the player's last direction to select idle animations
        if player.state == PlayerState::Idle {
            let indices = match player.last_direction {
                Some(Direction::RU) => player.idle_animations.up.clone(),
                Some(Direction::LU) => player.idle_animations.down.clone(),
                Some(Direction::LD) => player.idle_animations.left.clone(),
                Some(Direction::RD) => player.idle_animations.right.clone(),
                None => player.idle_animations.down.clone(), // Default idle direction
            };

            if sprite.index < indices.first || sprite.index > indices.last {
                sprite.index = indices.first;
            }

            // Update sprite index for idle animation
            if timer.just_finished() {
                sprite.index = if sprite.index == indices.last {
                    indices.first
                } else {
                    sprite.index + 1
                };
            }
        }
    }
}