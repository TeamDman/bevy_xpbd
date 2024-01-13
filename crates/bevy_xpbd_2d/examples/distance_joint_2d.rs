use bevy::prelude::*;
use bevy_xpbd_2d::{math::*, prelude::*};
use examples_common_2d::XpbdExamplePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            XpbdExamplePlugin,
            PhysicsDebugPlugin::default(),
        ))
        .add_event::<MovementAction>()
        .insert_resource(ClearColor(Color::rgb(0.05, 0.05, 0.1)))
        .insert_resource(SubstepCount(50))
        .insert_resource(Gravity(Vector::NEG_Y * 1000.0))
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                keyboard_input,
                movement.run_if(has_movement),
                apply_movement_damping,
            )
                .chain(),
        )
        .run();
}

#[derive(Component, Default, Reflect)]
struct Character;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    let square_sprite = Sprite {
        color: Color::rgb(0.2, 0.7, 0.9),
        custom_size: Some(Vec2::splat(50.0)),
        ..default()
    };

    let anchor = commands
        .spawn((
            SpriteBundle {
                sprite: square_sprite.clone(),
                ..default()
            },
            RigidBody::Kinematic,
            Character,
        ))
        .id();

    let object = commands
        .spawn((
            SpriteBundle {
                sprite: square_sprite,
                transform: Transform::from_xyz(100.0, 0.0, 0.0),
                ..default()
            },
            RigidBody::Dynamic,
            MassPropertiesBundle::new_computed(&Collider::cuboid(50.0, 50.0), 1.0),
        ))
        .id();

    commands.spawn(
        DistanceJoint::new(anchor, object)
            .with_local_anchor_1(Vector::ZERO)
            .with_local_anchor_2(Vector::ZERO)
            .with_rest_length(100.0)
            .with_linear_velocity_damping(0.1)
            .with_angular_velocity_damping(1.0)
            .with_limits(50.0, 150.0)
            .with_compliance(0.00000001),
    );
}

#[derive(Event, Debug, Reflect)]
pub enum MovementAction {
    Velocity(Vector),
    Stop,
}

// use velocity
fn keyboard_input(
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
        movement_event_writer.send(MovementAction::Velocity(direction));
    }
}

fn has_movement(mut reader: EventReader<MovementAction>) -> bool {
    reader.read().next().is_some()
}
fn movement(
    time: Res<Time>,
    mut movement_event_reader: EventReader<MovementAction>,
    mut controllers: Query<&mut LinearVelocity, With<Character>>,
) {
    let delta_time = time.delta_seconds_f64().adjust_precision();
    for event in movement_event_reader.read() {
        for mut linear_velocity in &mut controllers {
            match event {
                MovementAction::Stop => {
                    linear_velocity.x = 0.0;
                    linear_velocity.y = 0.0;
                }
                MovementAction::Velocity(direction) => {
                    let movement_acceleration = 2000.0;
                    linear_velocity.x += direction.x * movement_acceleration * delta_time;
                    linear_velocity.y += direction.y * movement_acceleration * delta_time;
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn apply_movement_damping(
    mut query: Query<
        (&mut LinearVelocity, &mut AngularVelocity),
        (With<Character>, Without<Sleeping>),
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
