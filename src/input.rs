use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::render::camera::Camera;
use bevy::math::Vec3A;

#[derive(Debug)]
pub struct InputState{
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    q: bool,
    e: bool,
    shift: bool,
    ctrl: bool,
}

pub fn set_up(
    mut commands: Commands,
) {
    commands.insert_resource(InputState { w: false,a: false,s: false,d: false,q: false,e: false, shift: false, ctrl: false})
}

pub fn keyboard_events(
    mut key_evr: EventReader<KeyboardInput>,
    mut cameras: Query<&mut GlobalTransform, With<Camera>>,
    mut state: ResMut<InputState>
) {
    use bevy::input::ElementState;

    for ev in key_evr.iter() {
        match ev.state {
            ElementState::Pressed => {
                match ev.key_code {
                    Some(KeyCode::W) => state.w = true,
                    Some(KeyCode::A) => state.a = true,
                    Some(KeyCode::S) => state.s = true,
                    Some(KeyCode::D) => state.d = true,
                    Some(KeyCode::E) => state.e = true,
                    Some(KeyCode::Q) => state.q = true,
                    Some(KeyCode::LShift) => state.shift = true,
                    Some(KeyCode::LControl) => state.ctrl = true,
                    _ => ()
                }
            }
            ElementState::Released => {
                match ev.key_code {
                    Some(KeyCode::W) => state.w = false,
                    Some(KeyCode::A) => state.a = false,
                    Some(KeyCode::S) => state.s = false,
                    Some(KeyCode::D) => state.d = false,
                    Some(KeyCode::E) => state.e = false,
                    Some(KeyCode::Q) => state.q = false,
                    Some(KeyCode::LShift) => state.shift = false,
                    Some(KeyCode::LControl) => state.ctrl = false,
                    _ => ()
                }
            }
        }
    }
    
    let mut x = 0.0;
    let mut z = 0.0;
    let mut r = 0.0;
    let mut y = 0.0;
    
    if state.w {z -= 0.3;}
    if state.s {z += 0.3;}
    
    if state.shift {y += 0.1;}
    if state.ctrl {y -= 0.1;}
    
    if state.a {x -= 0.3;}
    if state.d {x += 0.3;}
    
    if state.q {r += 0.05;}
    if state.e {r -= 0.05;}
    
    let q = Quat::from_rotation_y(r);
    
    for mut c in cameras.iter_mut() {
        let r = c.rotation.mul_vec3a(Vec3A::new(x,0.0,z));
        c.translation.x += r.x;
        c.translation.z += r.z;
        c.translation.y += y;
        //c.rotation = c.rotation.mul_quat(q);
        c.rotation = q.mul_quat(c.rotation)
    }
    
    
}