use super::*;
use bevy::ecs::system::RunSystemOnce;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};
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
    world.init_resource::<CamRig>();
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
fn movement_follows_the_camera_yaw_without_restoring_strafing() {
    for (key, direction) in [
        (KeyCode::KeyW, Vec3::NEG_X),
        (KeyCode::KeyA, Vec3::Z),
        (KeyCode::ArrowDown, Vec3::X),
        (KeyCode::ArrowRight, Vec3::NEG_Z),
    ] {
        let (mut world, hero) = moving_hero(&[key]);
        world.resource_mut::<CamRig>().yaw = std::f32::consts::FRAC_PI_2;
        world.run_system_once(drive_hero).unwrap();
        let transform = world.get::<Transform>(hero).unwrap();
        assert!((transform.translation.normalize() - direction).length() < 1e-5);
        assert!((transform.rotation * Vec3::NEG_Z - direction).length() < 1e-5);
        assert_eq!(
            world.get::<MovementState>(hero).unwrap().flags,
            move_flags::FORWARD
        );
    }
}

#[test]
fn diagonals_have_the_same_speed_as_a_single_direction() {
    let distance = |keys: &[KeyCode]| {
        let (mut world, hero) = moving_hero(keys);
        world.resource_mut::<CamRig>().yaw = 0.7;
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

fn look_world() -> (World, Entity) {
    let (mut world, _) = camera_world();
    let mut window = Window {
        focused: true,
        ..default()
    };
    window.set_cursor_position(Some(Vec2::new(200.0, 150.0)));
    let window = world
        .spawn((window, CursorOptions::default(), PrimaryWindow))
        .id();
    (world, window)
}

fn look_frame(world: &mut World, delta: Vec2) {
    world.resource_mut::<AccumulatedMouseMotion>().delta = delta;
    world.run_system_once(look_input).unwrap();
    world.resource_mut::<ButtonInput<KeyCode>>().clear();
}

fn assert_cursor_released(world: &World, window: Entity) {
    let cursor = world.get::<CursorOptions>(window).unwrap();
    assert_eq!(cursor.grab_mode, CursorGrabMode::None);
    assert!(cursor.visible);
    assert!(!world.resource::<CamRig>().looking);
}

#[test]
fn mouse_motion_orbits_without_holding_a_button_and_capture_skips_old_motion() {
    let (mut world, window) = look_world();
    let initial = (
        world.resource::<CamRig>().yaw,
        world.resource::<CamRig>().pitch,
    );
    look_frame(&mut world, Vec2::new(200.0, 100.0));
    assert!(world.resource::<CamRig>().looking);
    assert_eq!(
        (
            world.resource::<CamRig>().yaw,
            world.resource::<CamRig>().pitch
        ),
        initial
    );
    let cursor = world.get::<CursorOptions>(window).unwrap();
    assert_eq!(cursor.grab_mode, CursorGrabMode::Locked);
    assert!(!cursor.visible);
    assert!(!world
        .resource::<ButtonInput<MouseButton>>()
        .any_pressed([MouseButton::Left, MouseButton::Right]));
    look_frame(&mut world, Vec2::new(40.0, 20.0));
    let rig = world.resource::<CamRig>();
    assert!((rig.yaw - initial.0).abs() > 0.01);
    assert!((rig.pitch - initial.1).abs() > 0.01);
}

#[test]
fn looking_around_does_not_turn_a_stationary_hero() {
    let (mut world, _) = look_world();
    let initial_rotation = Quat::from_rotation_y(0.6);
    let hero = world
        .spawn((
            Hero { vy: 0.0, air: 0.0 },
            Transform::from_rotation(initial_rotation),
            MovementState::default(),
        ))
        .id();
    look_frame(&mut world, Vec2::ZERO);
    look_frame(&mut world, Vec2::new(100.0, 20.0));
    world.run_system_once(drive_hero).unwrap();
    let hero = world.get::<Transform>(hero).unwrap();
    assert!((hero.rotation * Vec3::NEG_Z - initial_rotation * Vec3::NEG_Z).length() < 1e-5);
    assert_eq!(hero.translation, Vec3::ZERO);
    assert!(world.resource::<CamRig>().yaw.abs() > 0.1);
}

#[test]
fn menus_alt_focus_loss_and_death_release_the_cursor() {
    for phase in [Phase::Menu, Phase::Paused, Phase::LevelUp] {
        let (mut world, window) = look_world();
        look_frame(&mut world, Vec2::ZERO);
        world.insert_resource(State::new(phase));
        look_frame(&mut world, Vec2::new(500.0, 500.0));
        assert_cursor_released(&world, window);
        assert_eq!(world.resource::<CamRig>().yaw, 0.0);
    }
    for key in [KeyCode::AltLeft, KeyCode::AltRight, KeyCode::Escape] {
        let (mut world, window) = look_world();
        look_frame(&mut world, Vec2::ZERO);
        world.resource_mut::<ButtonInput<KeyCode>>().press(key);
        look_frame(&mut world, Vec2::new(500.0, 500.0));
        assert_cursor_released(&world, window);
        assert_eq!(world.resource::<CamRig>().yaw, 0.0);
    }
    let (mut world, window) = look_world();
    look_frame(&mut world, Vec2::ZERO);
    world.get_mut::<Window>(window).unwrap().focused = false;
    look_frame(&mut world, Vec2::new(500.0, 500.0));
    assert_cursor_released(&world, window);
    let (mut world, window) = look_world();
    look_frame(&mut world, Vec2::ZERO);
    world.resource_mut::<Run>().dead_t = Some(0.0);
    look_frame(&mut world, Vec2::new(500.0, 500.0));
    assert_cursor_released(&world, window);
}

#[test]
fn open_overlays_release_the_pointer_even_while_the_run_keeps_playing() {
    for overlay in 0..3 {
        let (mut world, window) = look_world();
        look_frame(&mut world, Vec2::ZERO);
        match overlay {
            0 => {
                world.init_resource::<super::super::options::OptionsUi>();
                world
                    .resource_mut::<super::super::options::OptionsUi>()
                    .open = true;
            }
            1 => {
                world.init_resource::<super::super::charpanel::CharPanel>();
                world
                    .resource_mut::<super::super::charpanel::CharPanel>()
                    .open = true;
            }
            _ => {
                world
                    .resource_mut::<super::super::worldmap::WorldMapUi>()
                    .open = true
            }
        }
        look_frame(&mut world, Vec2::new(500.0, 500.0));
        assert_cursor_released(&world, window);
        assert_eq!(world.resource::<CamRig>().yaw, 0.0);
    }
}

#[test]
fn releasing_restores_the_pointer_and_recapture_does_not_jump() {
    let (mut world, window) = look_world();
    let original_cursor = world.get::<Window>(window).unwrap().cursor_position();
    look_frame(&mut world, Vec2::ZERO);
    look_frame(&mut world, Vec2::new(40.0, 10.0));
    let before = (
        world.resource::<CamRig>().yaw,
        world.resource::<CamRig>().pitch,
    );
    world
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(600.0, 300.0)));
    world
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::AltLeft);
    look_frame(&mut world, Vec2::new(500.0, 500.0));
    assert_cursor_released(&world, window);
    assert_eq!(
        world.get::<Window>(window).unwrap().cursor_position(),
        original_cursor
    );
    world
        .resource_mut::<ButtonInput<KeyCode>>()
        .release(KeyCode::AltLeft);
    look_frame(&mut world, Vec2::new(500.0, 500.0));
    assert!(world.resource::<CamRig>().looking);
    assert_eq!(
        (
            world.resource::<CamRig>().yaw,
            world.resource::<CamRig>().pitch
        ),
        before
    );
    look_frame(&mut world, Vec2::new(20.0, 0.0));
    assert!((world.resource::<CamRig>().yaw - before.0).abs() > 0.01);
}

#[test]
fn pitch_is_clamped_and_mouse_sensitivity_and_inversion_are_respected() {
    let orbit = |sensitivity: f32, invert: bool, delta: Vec2| {
        let (mut world, _) = look_world();
        {
            let mut prefs = world.resource_mut::<super::super::prefs::Prefs>();
            prefs.camera_sensitivity = sensitivity;
            prefs.invert_pitch = invert;
        }
        look_frame(&mut world, Vec2::ZERO);
        let before = (
            world.resource::<CamRig>().yaw,
            world.resource::<CamRig>().pitch,
        );
        look_frame(&mut world, delta);
        let rig = world.resource::<CamRig>();
        (
            (rig.yaw - before.0).sin().atan2((rig.yaw - before.0).cos()),
            rig.pitch - before.1,
            rig.pitch,
        )
    };
    let normal = orbit(1.0, false, Vec2::new(20.0, 10.0));
    let sensitive = orbit(2.0, false, Vec2::new(20.0, 10.0));
    let inverted = orbit(1.0, true, Vec2::new(20.0, 10.0));
    assert!(normal.0.abs() > 0.01 && normal.1.abs() > 0.01);
    assert!((sensitive.0 - 2.0 * normal.0).abs() < 1e-5);
    assert!((sensitive.1 - 2.0 * normal.1).abs() < 1e-5);
    assert!((inverted.1 + normal.1).abs() < 1e-5);
    assert!((orbit(1.0, false, Vec2::new(0.0, 10000.0)).2 - 1.45).abs() < 1e-5);
    assert!((orbit(1.0, false, Vec2::new(0.0, -10000.0)).2 + 0.25).abs() < 1e-5);
}

#[test]
fn camera_placement_does_not_consume_mouse_or_keyboard_input() {
    let (mut world, camera) = camera_world();
    world.run_system_once(place_camera).unwrap();
    let before = *world.get::<Transform>(camera).unwrap();
    world
        .resource_mut::<ButtonInput<MouseButton>>()
        .press(MouseButton::Left);
    world.resource_mut::<AccumulatedMouseMotion>().delta = Vec2::new(80.0, 40.0);
    world
        .resource_mut::<ButtonInput<KeyCode>>()
        .press(KeyCode::ArrowLeft);
    world.run_system_once(place_camera).unwrap();
    let after = world.get::<Transform>(camera).unwrap();
    assert!((before.translation - after.translation).length() < 1e-5);
    assert!((before.rotation * Vec3::NEG_Z - after.rotation * Vec3::NEG_Z).length() < 1e-5);
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

#[test]
fn captured_mouse_positions_cannot_hover_hud_buttons() {
    let (mut world, window) = look_world();
    look_frame(&mut world, Vec2::ZERO);
    assert!(world
        .get::<Window>(window)
        .unwrap()
        .cursor_position()
        .is_none());
    // Winit may still report a CursorMoved event while grabbed.
    world
        .get_mut::<Window>(window)
        .unwrap()
        .set_cursor_position(Some(Vec2::new(400.0, 300.0)));
    look_frame(&mut world, Vec2::new(1.0, 1.0));
    assert!(world
        .get::<Window>(window)
        .unwrap()
        .cursor_position()
        .is_none());
}

#[test]
fn queued_pause_and_resume_are_honored_before_the_state_transition() {
    let (mut world, window) = look_world();
    look_frame(&mut world, Vec2::ZERO);
    world.insert_resource(NextState::Pending(Phase::Paused));
    look_frame(&mut world, Vec2::new(500.0, 500.0));
    assert_cursor_released(&world, window);
    assert_eq!(world.resource::<CamRig>().yaw, 0.0);
    world.insert_resource(State::new(Phase::Paused));
    world.insert_resource(NextState::Pending(Phase::Playing));
    look_frame(&mut world, Vec2::new(500.0, 500.0));
    assert!(world.resource::<CamRig>().looking);
    assert_eq!(world.resource::<CamRig>().yaw, 0.0);
}
