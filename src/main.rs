use bevy::prelude::*;

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7,0.7,0.7).into()),
    });
}

struct SnakeHead;
struct Materials {
    head_material: Handle<ColorMaterial>,
}

fn spawn_snake(mut commands: Commands, materials: Res<Materials>) {
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.head_material.clone(),
            sprite: Sprite::new(Vec2::new(10.0,10.0)),
            ..Default::default()
        })
        .insert(SnakeHead);
}

// Remember that underscore means a variable isn't used
// Note that we're using a mutable iterator, an iterator over mutable things
fn snake_movement(mut head_positions: Query<(&SnakeHead, &mut Transform)>) {
    for(_head, mut transform) in head_positions.iter_mut() {
        transform.translation.y += 1.;
    }
}

fn main() {
    App::build()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_snake.system()))
        .add_system(snake_movement.system())
        .add_plugins(DefaultPlugins)
        .run();
}
