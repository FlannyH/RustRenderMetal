#![allow(dead_code)]
#![allow(clippy::needless_return)]

use std::{path::Path, collections::HashMap, time::Instant};
use glam::{Vec3, Quat, Vec2};
use graphics::{Renderer, ModelQueueEntry};
use metal::objc::rc::autoreleasepool;
use structs::Transform;
use winit::{event::{Event, WindowEvent, VirtualKeyCode, DeviceEvent, MouseButton}, event_loop::ControlFlow};

mod material;
mod mesh;
mod texture;
mod structs;
mod helpers;
mod graphics;

// Credits to https://github.com/gfx-rs/metal-rs/blob/master/examples/window/main.rs for the base structure
fn main() {
    // Create a window
    let event_loop = winit::event_loop::EventLoop::new();
    let res = winit::dpi::LogicalSize::new(1280, 720);
    let window = winit::window::WindowBuilder::new()
        .with_inner_size(res)
        .with_title("RustRenderMetal".to_string())
        .build(&event_loop)
        .unwrap();

    // Initialize renderer
    let mut renderer = Renderer::new(&window);
    // Load the Metal library file
    renderer.load_library("metal/shaders/hello_triangle.metallib");
    renderer.prepare_pipeline_state("hello_triangle_vertex", "hello_triangle_fragment");

    let model_suzanne = renderer.load_model(Path::new("./assets/suzanne.gltf")).unwrap();
    let model_gun = renderer.load_model(Path::new("./assets/sub_nivis_gun.gltf")).unwrap();

    let mut camera = Transform {
        translation: Vec3 {x: 0.0, y: 0.0, z: 0.5},
        rotation: Quat::IDENTITY,
        scale: Vec3 {x: 1.0, y: 1.0, z: 1.0},
    };

    // Main loop
    let mut x = 0.0;
    let mut time_curr = Instant::now();
    let mut time_prev = Instant::now();
    let mut key_held = HashMap::<VirtualKeyCode, bool>::new();
    let mut mouse_held = HashMap::<MouseButton, bool>::new();
    let camera_speed = 1.0;
    let mut delta_mouse_pos = Some(Vec2{x:0.0, y: 0.0});
    let mut camera_rotation = Vec3{x: 0.0, y: 0.0, z: 0.0};
    let mouse_sensitivity = -0.01;
    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent{event, ..} => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => renderer.resize_framebuffer(size.width, size.height),
                    WindowEvent::KeyboardInput { device_id: _, input, is_synthetic: _ } => {
                        match input.state {
                            winit::event::ElementState::Pressed => {
                                if input.virtual_keycode.is_some() {
                                    key_held.insert(input.virtual_keycode.unwrap(), true);
                                }
                            },
                            winit::event::ElementState::Released => {
                                if input.virtual_keycode.is_some() {
                                    key_held.insert(input.virtual_keycode.unwrap(), false);
                                }
                            }
                        };
                    },
                    WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                        match state {
                            winit::event::ElementState::Pressed => mouse_held.insert(button, true),
                            winit::event::ElementState::Released => mouse_held.insert(button, false)
                        };
                    },
                    _ => (),
                }
                Event::DeviceEvent{ device_id: _, event } => if let DeviceEvent::MouseMotion { delta } = event {
                    delta_mouse_pos = Some(Vec2{x: delta.0 as f32, y: delta.1 as f32});
                },
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    time_prev = time_curr;
                    time_curr = Instant::now();
                    let delta_time = (time_curr - time_prev).as_secs_f32();
                    x += delta_time * 2.0;
                    if *key_held.entry(VirtualKeyCode::D).or_insert(false) {camera.translation += delta_time * camera_speed * camera.right();}
                    if *key_held.entry(VirtualKeyCode::A).or_insert(false) {camera.translation -= delta_time * camera_speed * camera.right();}
                    if *key_held.entry(VirtualKeyCode::W).or_insert(false) {camera.translation += delta_time * camera_speed * camera.forward();}
                    if *key_held.entry(VirtualKeyCode::S).or_insert(false) {camera.translation -= delta_time * camera_speed * camera.forward();}
                    if *key_held.entry(VirtualKeyCode::Space).or_insert(false) {camera.translation += delta_time * camera_speed * camera.up();}
                    if *key_held.entry(VirtualKeyCode::LShift).or_insert(false) {camera.translation -= delta_time * camera_speed * camera.up();}
                    if let Some(delta_mouse) = delta_mouse_pos {
                        if *mouse_held.entry(MouseButton::Right).or_insert(false) {
                            camera_rotation.x += delta_mouse.x * mouse_sensitivity;
                            camera_rotation.y += delta_mouse.y * mouse_sensitivity;
                            delta_mouse_pos = None;
                        }
                    }
                    camera.rotation = Quat::from_euler(glam::EulerRot::YXZ, camera_rotation.x, camera_rotation.y, camera_rotation.z);
                    renderer.update_camera(&camera);
                    renderer.begin_frame();
                    renderer.draw_model(ModelQueueEntry{
                        model_id: model_gun,
                        transform: Transform { 
                            translation: Vec3{x: 0.0, y: 0.0, z: 0.0}, 
                            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, x, 0.0), 
                            scale: Vec3{x: 1.0, y: 1.0, z: 1.0} }
                    });
                    renderer.draw_model(ModelQueueEntry{
                        model_id: model_suzanne,
                        transform: Transform { 
                            translation: Vec3{x: 0.5, y: 0.0, z: 0.0}, 
                            rotation: Quat::from_euler(glam::EulerRot::XYZ, 0.0, x, 0.0), 
                            scale: Vec3{x: 0.1, y: 0.1, z: 0.1} }
                    });
                    renderer.end_frame();
                    window.request_redraw();
                }
                _ => {}
            }
        });
    });
}
