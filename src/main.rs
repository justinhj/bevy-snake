use bevy::prelude::*;
use bevy::render::pass::ClearColor;
use rand::prelude::random;
use bevy::core::FixedTimestep;

const ARENA_WIDTH: u32 = 10;
const ARENA_HEIGHT: u32 = 10;

// System labels
#[derive(SystemLabel,Debug,Hash,PartialEq,Eq,Clone)]
pub enum SnakeMovement {
    Input,
    Movement,
    Eating,
    Growth,
}


#[derive(PartialEq,Copy,Clone,Debug)]
enum Direction {
    Left,
    Right,
    Up,
    Down,
}

impl Direction {
    pub fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Up => Self::Down,
            Self::Down => Self::Up,
        }
    }
}

#[derive(Default)]
struct LastTailPosition(Option<Position>);

#[derive(Default,Copy,Clone,Eq,PartialEq,Hash,Debug)]
struct Position {
    x: i32,
    y: i32,
}

struct Size {
    width: f32,
    height: f32,
}

impl Size {
    pub fn square(x: f32) -> Self {
        Self {
            width: x,
            height: x,
        }
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.insert_resource(Materials {
        head_material: materials.add(Color::rgb(0.7,0.7,0.7).into()),
        segment_material: materials.add(Color::rgb(0.3,0.3,0.3).into()),
        food_material: materials.add(Color::rgb(1.,0.,1.).into()),
    });
}

struct SnakeHead {
    direction: Direction,
    last_input: Direction,
}

struct GrowthEvent;
struct GameOverEvent;
struct SnakeSegment;

#[derive(Default)]
struct SnakeSegments(Vec<Entity>);

struct Food;

struct Materials {
    head_material: Handle<ColorMaterial>,
    segment_material: Handle<ColorMaterial>,
    food_material: Handle<ColorMaterial>,
}

fn food_spawner(
    mut commands: Commands,
    materials: Res<Materials>,
    segment_positions: Query<&Position, With<SnakeSegment>>) {

    let mut x: i32;
    let mut y: i32;

    while {
        x = (random::<f32>() * ARENA_WIDTH as f32) as i32;
        y = (random::<f32>() * ARENA_HEIGHT as f32) as i32;

        !segment_positions.iter().all (|seg| !(seg.x == x && seg.y == y))

    } {}

    commands
        .spawn_bundle(SpriteBundle {
            material: materials.food_material.clone(),
            ..Default::default()
        })
        .insert(Food)
        .insert(Position {x, y })
        .insert(Size::square(0.8));
}

fn spawn_snake(mut commands: Commands, materials: Res<Materials>, mut segments: ResMut<SnakeSegments>) {
    segments.0 = vec![
        commands
            .spawn_bundle(SpriteBundle {
                material: materials.head_material.clone(),
                sprite: Sprite::new(Vec2::new(10.0,10.0)),
                ..Default::default()
            })
            .insert(SnakeHead{direction: Direction::Up, last_input: Direction::Up})
            .insert(SnakeSegment)
            .insert(Position { x: 3, y: 3 })
            .insert(Size::square(0.8))
            .id(),
            spawn_segment(
                commands,
                &materials.segment_material,
                Position { x: 3, y: 2 },
                ),
    ];
}

fn size_scaling(windows: Res<Windows>, mut q: Query<(&Size, &mut Sprite)>) {
    let window = windows.get_primary().unwrap();
    for (sprite_size,mut sprite) in q.iter_mut() {
        sprite.size = Vec2::new(
            sprite_size.width / ARENA_WIDTH as f32 * window.width() as f32,
            sprite_size.height / ARENA_HEIGHT as f32 * window.height() as f32,
            )
    }
}

fn position_translation(windows: Res<Windows>, mut q: Query<(&Position, &mut Transform)>) {
    fn convert(pos: f32, bound_window: f32, bound_game: f32) -> f32 {
        let tile_size = bound_window / bound_game;
        pos / bound_game * bound_window - (bound_window / 2.) + (tile_size / 2.)
    }

    let window = windows.get_primary().unwrap();
    for (pos, mut transform) in q.iter_mut() {
        transform.translation = Vec3::new(
            convert(pos.x as f32, window.width() as f32, ARENA_WIDTH as f32),
            convert(pos.y as f32, window.height() as f32, ARENA_HEIGHT as f32),
            0.,
            )
    }
}

fn snake_eating(mut commands: Commands,
                mut growth_writer: EventWriter<GrowthEvent>,
                food_positions: Query<(Entity,&Position), With<Food>>,
                head_positions: Query<&Position, With<SnakeHead>>) {
    for head_pos in head_positions.iter() {
        for (ent, food_position) in food_positions.iter() {
            if food_position == head_pos {
                commands.entity(ent).despawn();
                growth_writer.send(GrowthEvent);
            }
        }
    }
}   

fn snake_movement_input(
    keyboard_input: Res<Input<KeyCode>>, 
    mut heads: Query<&mut SnakeHead>) {
  if let Some(mut head) = heads.iter_mut().next() {
        let dir: Direction = if keyboard_input.pressed(KeyCode::Left) {
            // println!("direction left");
            Direction::Left
        } else if keyboard_input.pressed(KeyCode::Down) {
            // println!("direction down");
            Direction::Down
        } else if keyboard_input.pressed(KeyCode::Up) {
            // println!("direction up");
            Direction::Up
        } else if keyboard_input.pressed(KeyCode::Right) {
            // println!("direction right");
            Direction::Right
        } else {
            // println!("direction {:?}", head.direction);
            head.last_input
        };
        head.last_input = dir;
    }
}

fn snake_movement(segments: ResMut<SnakeSegments>, 
                  mut last_tail_position: ResMut<LastTailPosition>,
                  mut heads: Query<(Entity, &mut SnakeHead)>,
                  mut positions: Query<&mut Position>, 
                  mut game_over_writer: EventWriter<GameOverEvent>,
                  ) {
    if let Some((head_entity, mut head)) = heads.iter_mut().next() {
        let segment_positions = segments.0
            .iter()
            .map(|e| *positions.get_mut(*e).unwrap())
            .collect::<Vec<Position>>();

        let mut head_pos = positions.get_mut(head_entity).unwrap();
        
        // println!("Pre move {:?} dir {:?} {:?}", head_pos, head.direction, head.last_input);

        if head.direction != head.last_input.opposite() {
            head.direction = head.last_input;
        }

        match &head.direction {
            Direction::Left => {
                head_pos.x -= 1;
            }
            Direction::Right => {
                head_pos.x += 1;
            }
            Direction::Up => {
                head_pos.y += 1;
            }
            Direction::Down => {
                head_pos.y -= 1;
            }
        };


        // println!("Post move {:?} dir {:?} {:?}", head_pos, head.direction, head.last_input);

        // Handle hitting the wall
        if head_pos.x < 0
            || head_pos.y < 0
            || head_pos.x as u32 >= ARENA_WIDTH
            || head_pos.y as u32 >= ARENA_HEIGHT {
            game_over_writer.send(GameOverEvent);
        }
        // Handle hitting ourselves
        if segment_positions.contains(&head_pos) {
            // println!("game over");
            game_over_writer.send(GameOverEvent);
        }
        segment_positions
            .iter()
            .zip(segments.0.iter().skip(1))
            .for_each(|(pos,segment)| {
                *positions.get_mut(*segment).unwrap() = *pos;
            });
        last_tail_position.0 = Some(*segment_positions.last().unwrap());
    }
}

fn snake_growth(commands: Commands,
    last_tail_position: Res<LastTailPosition>,
    mut segments: ResMut<SnakeSegments>,
    mut growth_reader: EventReader<GrowthEvent>,
    materials: Res<Materials>,
) {
    if growth_reader.iter().next().is_some() {
        segments.0.push(spawn_segment(
            commands,
            &materials.segment_material,
            last_tail_position.0.unwrap(),
        ));
    }
}

fn spawn_segment(mut commands: Commands, material: &Handle<ColorMaterial>, position: Position) -> Entity {
    commands
        .spawn_bundle(SpriteBundle {
            material: material.clone(),
            ..Default::default()
        })
        .insert(SnakeSegment)
        .insert(position)
        .insert(Size::square(0.65))
        .id()
}

fn game_over(
    mut commands: Commands,
    mut reader: EventReader<GameOverEvent>,
    materials: Res<Materials>,
    segments_res: ResMut<SnakeSegments>,
    food: Query<Entity, With<Food>>,
    segments: Query<Entity, With<SnakeSegment>>,
) {
    if reader.iter().next().is_some() {
        for ent in segments.iter().chain(food.iter()) {
            commands.entity(ent).despawn();
        }
        spawn_snake(commands, materials, segments_res)
    }
}

fn main() {
    App::build()
        .insert_resource(WindowDescriptor {
            title: "Bevy-Snake".to_string(),
            width: 500.,
            height: 500.,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04,0.04,0.04)))
        .insert_resource(SnakeSegments::default())
        .insert_resource(LastTailPosition::default())
        .add_event::<GameOverEvent>()
        .add_event::<GrowthEvent>()
        .add_startup_system(setup.system())
        .add_startup_stage("game_setup", SystemStage::single(spawn_snake.system()))
        .add_system(
            snake_movement_input
                .system()
                .label(SnakeMovement::Input)
                .before(SnakeMovement::Movement),
            )
        .add_system(game_over.system().after(SnakeMovement::Movement))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.150))
                .with_system(
                    snake_growth
                        .system()
                        .label(SnakeMovement::Growth)
                        .after(SnakeMovement::Eating),
                )
                .with_system(snake_movement.system().label(SnakeMovement::Movement))
                .with_system(snake_eating.system()
                             .label(SnakeMovement::Eating)
                            .after(SnakeMovement::Movement))
                )
        .add_system_set(SystemSet::new()
                        .with_run_criteria(FixedTimestep::step(1.0))
                        .with_system(food_spawner.system()))
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(position_translation.system())
                .with_system(size_scaling.system()))
        .add_plugins(DefaultPlugins)
        .run();
}
