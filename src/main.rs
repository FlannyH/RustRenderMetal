#![allow(dead_code)]
#![allow(clippy::needless_return)]

use std::path::Path;
use glam::{Vec3, Quat};
use graphics::{Renderer, ModelQueueEntry};
use metal::objc::rc::autoreleasepool;
use structs::Transform;
use winit::{event::{Event, WindowEvent}, event_loop::ControlFlow};

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

    let model_suzanne = renderer.load_model(Path::new("./assets/sub_nivis_gun.gltf")).unwrap();

    let mut camera = Transform {
        translation: Vec3 {x: 0.0, y: 0.0, z: -1.0},
        rotation: Quat::IDENTITY,
        scale: Vec3 {x: 1.0, y: 1.0, z: 1.0},
    };

    // Main loop
    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent{event, ..} => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => renderer.resize_framebuffer(size.width, size.height),
                    _ => (),
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    camera.translation.z += 0.01;
                    renderer.update_camera(&camera);
                    renderer.begin_frame();
                    renderer.draw_model(ModelQueueEntry{model_id: model_suzanne});
                    renderer.end_frame();
                    window.request_redraw();
                }
                _ => {}
            }
        });
    });
}
