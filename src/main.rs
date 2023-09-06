#![allow(dead_code)]
#![allow(clippy::needless_return)]

use std::{path::Path, hash::Hash};

use glam::{Vec2, Vec3, Vec4};
use graphics::Renderer;
use mesh::{Mesh, Model};
use metal::objc::rc::autoreleasepool;
use structs::Vertex;
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


    // Set up vertex buffer data for the triangle
    let mut mesh_triangle = Mesh {
        verts: vec![
            Vertex{ 
                position: Vec3{x: -0.5, y: -0.5, z: 0.0}, 
                color: Vec4{x: 1.0, y: 0.0, z: 0.0, w: 1.0},
                normal: Vec3{x: 0.0, y: 0.0, z: 0.0},
                tangent: Vec4{x: 0.0, y: 0.0, z: 0.0, w: 1.0},
                uv0: Vec2{x: 0.0, y: 0.0},
                uv1: Vec2{x: 0.0, y: 0.0},
            },
            Vertex{ 
                position: Vec3{x: 0.5, y: -0.5, z: 0.0}, 
                color: Vec4{x: 0.0, y: 1.0, z: 0.0, w: 1.0},
                normal: Vec3{x: 0.0, y: 0.0, z: 0.0},
                tangent: Vec4{x: 0.0, y: 0.0, z: 0.0, w: 1.0},
                uv0: Vec2{x: 0.0, y: 0.0},
                uv1: Vec2{x: 0.0, y: 0.0},
            },
            Vertex{ 
                position: Vec3{x: 0.0, y: 0.5, z: 0.0}, 
                color: Vec4{x: 0.0, y: 0.0, z: 1.0, w: 1.0},
                normal: Vec3{x: 0.0, y: 0.0, z: 0.0},
                tangent: Vec4{x: 0.0, y: 0.0, z: 0.0, w: 1.0},
                uv0: Vec2{x: 0.0, y: 0.0},
                uv1: Vec2{x: 0.0, y: 0.0},
            },
        ],
        buffer: None,
    };
    let mut model_suzanne = Model::load_gltf(Path::new("./assets/suzanne.gltf")).expect("Could not find suzanne.gltf");
    let mut mesh_suzanne = Mesh{ verts: Vec::new(), buffer: None };

    for (_key, value) in model_suzanne.meshes {
        mesh_suzanne = value;
    }

    renderer.upload_vertex_buffer(&mut mesh_suzanne);

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
                    renderer.render_frame(&mesh_suzanne);
                    window.request_redraw();
                }
                _ => {}
            }
        });
    });
}
