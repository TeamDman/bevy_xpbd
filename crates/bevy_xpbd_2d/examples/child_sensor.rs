use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_xpbd_2d::{math::*, prelude::*};
use examples_common_2d::XpbdExamplePlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, XpbdExamplePlugin))
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity(Vector::ZERO))
        .add_event::<MovementAction>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                keyboard_input1,
                keyboard_input2,
                apply_deferred,
                movement.run_if(has_movement), // don't mutably access the character if there is no movement
                apply_movement_damping,
                apply_pressure_plate_colour,
                update_text,
                log_events,
                update_ghosts,
            )
                .chain(),
        )
        .run();
}

#[derive(Component, Default, Reflect)]
struct Character;

#[derive(Component, Default, Reflect)]
struct PressurePlate;

#[derive(Component, Default, Reflect)]
struct Hand;

#[derive(Component, Default, Reflect)]
struct Ghost;

#[derive(Component, Default, Reflect)]
struct MyText;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut character = commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(
                    shape::Capsule {
                        radius: 12.5,
                        depth: 20.0,
                        ..default()
                    }
                    .into(),
                )
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.2, 0.7, 0.9))),
            transform: Transform::from_xyz(0.0, -100.0, 0.0),
            ..default()
        },
        Character,
        RigidBody::Dynamic,
        Collider::capsule(20.0, 12.5),
        Name::new("Character"),
    ));

    character.with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(0.0, 150.0, 0.0),
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(Vec2::new(100.0, 100.0)),
                    ..default()
                },
                ..default()
            },
            PressurePlate,
            Sensor,
            RigidBody::Static,
            Collider::cuboid(100.0, 100.0),
            Name::new("Pressure Plate"),
        ));

        parent.spawn((
            MaterialMesh2dBundle {
                mesh: meshes
                    .add(
                        shape::Circle {
                            radius: 25.0,
                            ..default()
                        }
                        .into(),
                    )
                    .into(),
                material: materials.add(ColorMaterial::from(Color::rgb(0.9, 0.7, 0.9))),
                transform: Transform::from_xyz(100.0, 0.0, 10.0),
                ..default()
            },
            Hand,
            RigidBody::Dynamic,
            Collider::ball(25.0),
            Name::new("Hand"),
        ));
    });

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(
                    shape::Capsule {
                        radius: 12.5,
                        depth: 20.0,
                        ..default()
                    }
                    .into(),
                )
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.7))),
            transform: Transform::from_xyz(0.0, -100.0, 0.0),
            ..default()
        },
        Character,
        Ghost,
        Name::new("Ghost Character"),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.7, 0.7, 0.7),
                custom_size: Some(Vec2::new(100.0, 100.0)),
                ..default()
            },
            ..default()
        },
        PressurePlate,
        Ghost,
        Sensor,
        Name::new("Ghost Pressure Plate"),
    ));

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes
                .add(
                    shape::Circle {
                        radius: 25.0,
                        ..default()
                    }
                    .into(),
                )
                .into(),
            material: materials.add(ColorMaterial::from(Color::rgb(0.7, 0.7, 0.7))),
            ..default()
        },
        Hand,
        Ghost,
        Name::new("Ghost Hand"),
    ));

    commands.spawn((
        TextBundle::from_section(
            "Waiting for text update",
            TextStyle {
                font_size: 16.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        MyText,
        Name::new("Character Velocity Text"),
    ));

    commands.spawn(Camera2dBundle::default());
}

#[derive(Event, Debug, Reflect)]
pub enum MovementAction {
    Character(Vector),
    Hand(Vector),
    Stop,
}

// use velocity
fn keyboard_input1(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time<Physics>>,
) {
    if time.is_paused() {
        return;
    }
    let space = keyboard_input.pressed(KeyCode::Space);
    if space {
        movement_event_writer.send(MovementAction::Stop);
        return;
    }

    let left = keyboard_input.any_pressed([KeyCode::A]);
    let right = keyboard_input.any_pressed([KeyCode::D]);
    let up = keyboard_input.any_pressed([KeyCode::W]);
    let down = keyboard_input.any_pressed([KeyCode::S]);
    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let direction = Vector::new(horizontal as Scalar, vertical as Scalar);
    if direction != Vector::ZERO {
        movement_event_writer.send(MovementAction::Character(direction));
    }
}
// use position offset
fn keyboard_input2(
    mut movement_event_writer: EventWriter<MovementAction>,
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time<Physics>>,
) {
    if time.is_paused() {
        return;
    }
    let left = keyboard_input.any_pressed([KeyCode::Left]);
    let right = keyboard_input.any_pressed([KeyCode::Right]);
    let up = keyboard_input.any_pressed([KeyCode::Up]);
    let down = keyboard_input.any_pressed([KeyCode::Down]);
    let horizontal = right as i8 - left as i8;
    let vertical = up as i8 - down as i8;
    let direction = Vector::new(horizontal as Scalar, vertical as Scalar);
    if direction != Vector::ZERO {
        movement_event_writer.send(MovementAction::Hand(direction));
    }
}

fn has_movement(mut reader: EventReader<MovementAction>) -> bool {
    reader.read().next().is_some()
}
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<&mut LinearVelocity, (With<Character>, Without<Hand>, Without<Ghost>)>,
    mut hands: Query<&mut LinearVelocity, (With<Hand>, Without<Character>, Without<Ghost>)>,
) {
    let delta_time = time.delta_seconds_f64().adjust_precision();
    for mut character_velocity in &mut controllers {
        for mut hand_velocity in &mut hands {
            for event in movement_event_reader.read() {
                match event {
                    MovementAction::Stop => {
                        character_velocity.x = 0.0;
                        character_velocity.y = 0.0;
                        hand_velocity.x = 0.0;
                        hand_velocity.y = 0.0;
                    }
                    MovementAction::Character(direction) => {
                        let movement_acceleration = 2000.0;
                        character_velocity.x += direction.x * movement_acceleration * delta_time;
                        character_velocity.y += direction.y * movement_acceleration * delta_time;
                    }
                    MovementAction::Hand(direction) => {
                        let movement_acceleration = 2000.0;
                        hand_velocity.x += direction.x * movement_acceleration * delta_time;
                        hand_velocity.y += direction.y * movement_acceleration * delta_time;
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn apply_movement_damping(
    mut query: Query<
        (&mut LinearVelocity, &mut AngularVelocity),
        (
            Or<(With<Character>, With<Hand>)>,
            Without<Sleeping>,
            Without<Ghost>,
        ),
    >,
    time: Res<Time<Physics>>,
) {
    if time.is_paused() {
        return;
    }
    let damping_factor = 0.95;
    for (mut linear_velocity, mut angular_velocity) in &mut query {
        linear_velocity.x *= damping_factor;
        if linear_velocity.x.abs() < 0.001 {
            linear_velocity.x = 0.0;
        }
        linear_velocity.y *= damping_factor;
        if linear_velocity.y.abs() < 0.001 {
            linear_velocity.y = 0.0;
        }
        angular_velocity.0 *= damping_factor;
        if angular_velocity.0.abs() < 0.001 {
            angular_velocity.0 = 0.0;
        }
    }
}

fn apply_pressure_plate_colour(
    mut query: Query<(&mut Sprite, &CollidingEntities), (With<PressurePlate>, Without<Ghost>)>,
) {
    for (mut sprite, colliding_entities) in &mut query {
        if colliding_entities.0.is_empty() {
            sprite.color = Color::rgb(0.2, 0.7, 0.9);
        } else {
            sprite.color = Color::rgb(0.9, 0.7, 0.2);
        }
    }
}

fn update_text(
    character_query: Query<
        (
            &LinearVelocity,
            Has<Sleeping>,
            &Transform,
            &GlobalTransform,
            &Position,
        ),
        (With<Character>, Without<Hand>, Without<Ghost>),
    >,
    hand_query: Query<
        (
            &LinearVelocity,
            Has<Sleeping>,
            &Transform,
            &GlobalTransform,
            &Position,
        ),
        (With<Hand>, Without<Character>, Without<Ghost>),
    >,
    pressure_plate_query: Query<
        (Has<Sleeping>, &Position, &Transform, &GlobalTransform),
        (With<PressurePlate>, Without<Ghost>),
    >,
    mut text_query: Query<&mut Text, With<MyText>>,
) {
    if let (
        Ok((c_vel, c_sleeping, c_trans, c_global_trans, c_pos)),
        Ok((h_vel, h_sleeping, h_trans, h_global_trans, h_pos)),
        Ok((p_sleeping, p_pos, p_trans, p_global_trans)),
    ) = (
        character_query.get_single(),
        hand_query.get_single(),
        pressure_plate_query.get_single(),
    ) {
        // text_query.single_mut().sections[0].value = format!(
        //     "Velocity: {:.4}, {:.4}\nCharacter sleeping:{}\nPressure plate sleeping: {}",
        //     velocity.x, velocity.y, character_sleeping, pressure_plate_sleeping
        // );
        text_query.single_mut().sections[0].value = format!(
            "c_vel: {:.4}, {:.4}\n\
            h_vel: {:.4}, {:.4}\n\
            c_sleeping: {}\n\
            h_sleeping: {}\n\
            p_sleeping: {}\n\
            c_pos: {:?}\n\
            h_pos: {:?}\n\
            p_pos: {:?}\n\
            c_trans: {:?}\n\
            h_trans: {:?}\n\
            p_trans: {:?}\n\
            c_global_trans: {:?}\n\
            h_global_trans: {:?}\n\
            p_global_trans: {:?}\n\
            ",
            c_vel.x,
            c_vel.y,
            h_vel.x,
            h_vel.y,
            c_sleeping,
            h_sleeping,
            p_sleeping,
            c_pos,
            h_pos,
            p_pos,
            c_trans.translation,
            h_trans.translation,
            p_trans.translation,
            c_global_trans.translation(),
            h_global_trans.translation(),
            p_global_trans.translation(),
        );
    }
}

fn log_events(mut started: EventReader<CollisionStarted>, mut ended: EventReader<CollisionEnded>) {
    // print out the started and ended events
    for event in started.read() {
        println!("CollisionStarted: {:?}", event);
    }
    for event in ended.read() {
        println!("CollisionEnded: {:?}", event);
    }
}

fn update_ghosts(
    character_query: Query<
        (&Position, &Rotation),
        (
            With<Character>,
            Without<Hand>,
            Without<PressurePlate>,
            Without<Ghost>,
        ),
    >,
    mut character_ghost_query: Query<
        (&mut Transform,),
        (
            With<Character>,
            Without<Hand>,
            Without<PressurePlate>,
            With<Ghost>,
        ),
    >,
    hand_query: Query<
        (&Position, &Rotation),
        (
            With<Hand>,
            Without<Character>,
            Without<PressurePlate>,
            Without<Ghost>,
        ),
    >,
    mut hand_ghost_query: Query<
        (&mut Transform,),
        (
            With<Hand>,
            Without<Character>,
            Without<PressurePlate>,
            With<Ghost>,
        ),
    >,
    pressure_plate_query: Query<
        (&Position, &Rotation),
        (
            With<PressurePlate>,
            Without<Character>,
            Without<Hand>,
            Without<Ghost>,
        ),
    >,
    mut pressure_plate_ghost_query: Query<
        (&mut Transform,),
        (
            With<PressurePlate>,
            Without<Character>,
            Without<Hand>,
            With<Ghost>,
        ),
    >,
) {
    if let (Ok(character), Ok(hand), Ok(pressure_plate)) = (
        character_query.get_single(),
        hand_query.get_single(),
        pressure_plate_query.get_single(),
    ) {
        let mut character_ghost = character_ghost_query.single_mut();
        character_ghost.0.translation = character.0.extend(0.0);
        character_ghost.0.rotation = Quat::from_rotation_z(character.1.as_radians());

        let mut hand_ghost = hand_ghost_query.single_mut();
        hand_ghost.0.translation = hand.0.extend(0.0);
        hand_ghost.0.rotation = Quat::from_rotation_z(hand.1.as_radians());

        let mut pressure_plate_ghost = pressure_plate_ghost_query.single_mut();
        pressure_plate_ghost.0.translation = pressure_plate.0.extend(0.0);
        pressure_plate_ghost.0.rotation = Quat::from_rotation_z(pressure_plate.1.as_radians());
    }
}
