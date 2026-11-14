use super::*;
use bevy::ecs::system::RunSystemOnce;
use std::time::Duration;

fn empty_world() -> World {
    let mut world = World::new();
    benilla_world::world_point::init_world_point_resources(&mut world);
    world.init_resource::<avian3d::collider_tree::ColliderTrees>();
    world.init_resource::<avian3d::prelude::PhysicsLengthUnit>();
    world.init_resource::<benilla_world::collision::MoverTraceExclusions>();
    world.init_resource::<benilla_app::survivors_host::player::Player>();
    world.init_resource::<ButtonInput<KeyCode>>();
    world.init_resource::<Run>();
    world.init_resource::<Selection>();
    world.init_resource::<super::super::route::Bounds>();
    world.insert_resource(State::new(Phase::Playing));
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_secs_f32(0.1));
    world.insert_resource(time);
    world
}

fn moving_hero(keys: &[KeyCode]) -> (World, Entity) {
    let mut world = empty_world();
    for &key in keys {
        world.resource_mut::<ButtonInput<KeyCode>>().press(key);
    }
    let hero = world
        .spawn((
            Hero { vy: 0.0, air: 0.0 },
            Transform::default(),
            MovementState::default(),
        ))
        .id();
    (world, hero)
}

#[test]
fn moving_left_or_down_faces_the_way_the_hero_runs() {
    for (key, direction) in [(KeyCode::KeyA, Vec3::NEG_X), (KeyCode::KeyS, Vec3::Z)] {
        let (mut world, hero) = moving_hero(&[key]);
        world.run_system_once(drive_hero).unwrap();
        let transform = world.get::<Transform>(hero).unwrap();
        let movement = world.get::<MovementState>(hero).unwrap();
        assert!(transform.translation.dot(direction) > 0.0);
        assert!((transform.translation.normalize() - direction).length() < 1e-5);
        assert!((transform.rotation * Vec3::NEG_Z - direction).length() < 1e-5);
        assert_eq!(movement.flags, move_flags::FORWARD);
        assert!(movement.speed > 0.0);
    }
}

#[test]
fn diagonals_have_the_same_speed_as_a_single_direction() {
    let distance = |keys: &[KeyCode]| {
        let (mut world, hero) = moving_hero(keys);
        world.run_system_once(drive_hero).unwrap();
        world.get::<Transform>(hero).unwrap().translation.length()
    };
    let straight = distance(&[KeyCode::KeyW]);
    assert!(straight > 0.0);
    assert!((distance(&[KeyCode::KeyW, KeyCode::KeyD]) - straight).abs() < 1e-5);
    assert!((distance(&[KeyCode::ArrowDown, KeyCode::ArrowLeft]) - straight).abs() < 1e-5);
    assert_eq!(distance(&[KeyCode::KeyW, KeyCode::KeyS]), 0.0);
    assert_eq!(distance(&[KeyCode::KeyA, KeyCode::ArrowRight]), 0.0);
}

#[test]
fn pausing_or_dying_stops_movement_and_the_running_animation() {
    for (phase, dead) in [
        (Phase::Paused, false),
        (Phase::LevelUp, false),
        (Phase::Playing, true),
    ] {
        let (mut world, hero) = moving_hero(&[KeyCode::KeyD]);
        world.insert_resource(State::new(phase));
        world.resource_mut::<Run>().dead_t = dead.then_some(0.0);
        {
            let mut movement = world.get_mut::<MovementState>(hero).unwrap();
            movement.speed = 7.0;
            movement.flags = move_flags::FORWARD;
        }
        world.run_system_once(drive_hero).unwrap();
        assert_eq!(
            world.get::<Transform>(hero).unwrap().translation,
            Vec3::ZERO
        );
        let movement = world.get::<MovementState>(hero).unwrap();
        assert_eq!(movement.speed, 0.0);
        assert_eq!(movement.flags, 0);
    }
}

fn camera_world() -> (World, Entity) {
    let mut world = empty_world();
    world.init_resource::<CamRig>();
    world.init_resource::<Messages<MouseWheel>>();
    world.init_resource::<ButtonInput<MouseButton>>();
    world.init_resource::<AccumulatedMouseMotion>();
    world.init_resource::<super::super::prefs::Prefs>();
    world.init_resource::<super::super::prefs::Shake>();
    world.init_resource::<super::super::worldmap::WorldMapUi>();
    world.init_resource::<super::super::worldmap::HubMapUi>();
    let mut time = Time::<Real>::default();
    time.advance_by(Duration::from_secs_f32(0.1));
    world.insert_resource(time);
    let camera = world.spawn((WorldCamera, Transform::default())).id();
    (world, camera)
}

#[test]
fn dragging_or_pressing_horizontal_arrows_does_not_turn_the_run_camera() {
    for button in [MouseButton::Left, MouseButton::Right] {
        let (mut world, camera) = camera_world();
        world.run_system_once(place_camera).unwrap();
        let before = *world.get::<Transform>(camera).unwrap();
        world
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(button);
        world.resource_mut::<AccumulatedMouseMotion>().delta = Vec2::new(80.0, 40.0);
        world
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::ArrowLeft);
        world.run_system_once(place_camera).unwrap();
        let after = world.get::<Transform>(camera).unwrap();
        assert!((before.translation - after.translation).length() < 1e-5);
        assert!((before.rotation * Vec3::NEG_Z - after.rotation * Vec3::NEG_Z).length() < 1e-5);
    }
}

#[test]
fn dragging_the_menu_preview_still_turns_it() {
    let (mut world, camera) = camera_world();
    world.insert_resource(State::new(Phase::Menu));
    world.run_system_once(place_camera).unwrap();
    let before = *world.get::<Transform>(camera).unwrap();
    let orbit_before = world.resource::<CamRig>().orbit;
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    world.resource_mut::<AccumulatedMouseMotion>().delta = Vec2::new(80.0, 10.0);
    world.run_system_once(place_camera).unwrap();
    let after = world.get::<Transform>(camera).unwrap();
    assert!(world.resource::<CamRig>().menu_drag);
    assert!((world.resource::<CamRig>().orbit - orbit_before).abs() > 0.1);
    assert!(before.rotation.angle_between(after.rotation) > 0.1);
}
