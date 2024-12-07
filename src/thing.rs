use rand::Rng;
use bevy::prelude::*;

use crate::{Money, Player};

#[derive(Component)]
pub struct ThingH {
    pub lifetime: Timer,
    pub direction: Option<Vec2>,
}
fn spawn_h(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    input: Res<ButtonInput<KeyCode>>,
    mut money: ResMut<Money>,
    player: Query<&Transform, With<Player>>,
) {
    if !input.just_pressed(KeyCode::Space) {
        return;
    }
    let player_transform = player.single();

    if money.0 >= 10.0 {
        money.0 -= 10.0;
        info!("Spent $10 on an h, remaining money: ${:}", money.0);

        let texture = asset_server.load("Items/Arrow.png");
        let place = *player_transform;
        commands.spawn((SpriteBundle {
            texture,
            transform: Transform::from_xyz(place.translation.x, place.translation.y, -1e-3).with_scale(place.scale),
            ..default()
        },
                        ThingH {
                            lifetime: Timer::from_seconds(2.0, TimerMode::Once),
                            direction: {
                                let angle = rand::thread_rng().gen_range(0.0..(2.0 * std::f32::consts::PI));
                                Some(Vec2::new(angle.cos(), angle.sin()).normalize())
                            },
                        }
        ));
    }
}
fn h_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut things_h: Query<(Entity, &mut ThingH, &mut Transform)>,
    mut money: ResMut<Money>,
) {
    for (h_entity, mut thing, mut transform) in &mut things_h {
        if let Some(direction) = thing.direction {
            let h_speed = 40.0; // Speed in units per second
            let movement = direction * h_speed * time.delta_seconds();
            transform.translation += movement.extend(0.0); // Extend Vec2 to Vec3
        }

        // Decrease lifetime
        thing.lifetime.tick(time.delta());
        if thing.lifetime.just_finished() {
            money.0 += 15.0;
            commands.entity(h_entity).despawn();
            info!("Pig sold for $15! Current Money: ${:?}", money.0);
        }
    }
}

pub struct ThingPlugin;
impl Plugin for ThingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, (spawn_h, h_lifetime));
    }
}