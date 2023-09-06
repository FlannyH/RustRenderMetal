use std::{fs::File, mem};

use cocoa::base::YES;
use cocoa::{appkit::NSView, base::id as cocoa_id};
use glam::{Vec2, Vec3, Vec4};
use metal::objc::rc::autoreleasepool;
use metal::{Device, MetalLayer, MTLPixelFormat, LibraryRef, DeviceRef, RenderPipelineState, RenderPipelineDescriptor, MTLResourceOptions, MTLLoadAction, RenderPassDescriptor, MTLClearColor, MTLStoreAction, MTLScissorRect, MTLPrimitiveType};
use winit::event::{Event, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::platform::macos::WindowExtMacOS;
use core_graphics_types::geometry::CGSize;


#[repr(C)]
#[derive(Debug)]
struct HelloTriangleVertex {
    position: Vec2,
    color: Vec3,
}

fn prepare_pipeline_state (
    device: &DeviceRef,
    library: &LibraryRef,
    vertex_shader_path: &str,
    fragment_shader_path: &str,
 ) -> RenderPipelineState {
    // Get compiled functions from the library
    let vertex_function = library.get_function(vertex_shader_path, None).unwrap();
    let fragment_function = library.get_function(fragment_shader_path, None).unwrap();

    // Create pipeline state descriptor - handles things like shader program, buffer to render to, blend mode, etc.
    let pipeline_state_desc = RenderPipelineDescriptor::new();
    pipeline_state_desc.set_vertex_function(Some(&vertex_function));
    pipeline_state_desc.set_fragment_function(Some(&fragment_function));

    let attachment = pipeline_state_desc.color_attachments().object_at(0).unwrap();
    attachment.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
    attachment.set_blending_enabled(false);
    return device.new_render_pipeline_state(&pipeline_state_desc).unwrap();
}

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

    // Create device
    let device = Device::system_default().expect("Could not create device.");

    // Create metal layer
    let layer = MetalLayer::new();
    layer.set_device(&device);
    layer.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
    layer.set_presents_with_transaction(false);

    // Create view - a sort of canvas where you draw graphics using Metal commands
    unsafe {
        let view = window.ns_view() as cocoa_id;
        view.setWantsLayer(YES);
        view.setLayer(mem::transmute(layer.as_ref()));
    }

    let drawable_size = window.inner_size();
    layer.set_drawable_size(CGSize::new(drawable_size.width as f64, drawable_size.height as f64));

    // Load the Metal library file
    let library = device.new_library_with_file("metal/shaders/hello_triangle.metallib").expect("Failed to load Metal library");

    // Initialize the pipeline states with functions from this library
    let hello_triangle_pipeline_state = prepare_pipeline_state(&device, &library, "hello_triangle_vertex", "hello_triangle_fragment");

    // Create command queue
    let command_queue = device.new_command_queue();

    // Set up vertex buffer data for the triangle
    let mut vertex_buffer_data = Vec::<HelloTriangleVertex>::new();
    // todo: make sure the winding order is correct
    vertex_buffer_data.push(HelloTriangleVertex{ position: Vec2{x: -0.5, y: -0.5}, color: Vec3{x: 1.0, y: 0.0, z: 0.0} });
    vertex_buffer_data.push(HelloTriangleVertex{ position: Vec2{x: 0.5, y: -0.5}, color: Vec3{x: 0.0, y: 1.0, z: 0.0} });
    vertex_buffer_data.push(HelloTriangleVertex{ position: Vec2{x: 0.0, y: 0.5}, color: Vec3{x: 0.0, y: 0.0, z: 1.0} });

    // Create the vertex buffer on the device
    let vertex_buffer = device.new_buffer_with_data(
        vertex_buffer_data.as_ptr() as *const _,
        (vertex_buffer_data.len() * mem::size_of::<HelloTriangleVertex>()) as u64,
        MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
    );
    println!("buffer is {} bytes long", vertex_buffer.length());

    // Main loop
    event_loop.run(move |event, _, control_flow| {
        autoreleasepool(|| {
            *control_flow = ControlFlow::Poll;

            match event {
                Event::WindowEvent{event, ..} => match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(size) => layer.set_drawable_size(CGSize::new(size.width as f64, size.height as f64)),
                    _ => (),
                }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    // Get the next framebuffer
                    let drawable = match layer.next_drawable() {
                        Some(drawable) => drawable,
                        None => return,
                    };

                    // Set up framebuffer
                    let render_pass_descriptor = RenderPassDescriptor::new();
                    let color_attachment = render_pass_descriptor.color_attachments().object_at(0).unwrap();
                    color_attachment.set_texture(Some(drawable.texture()));
                    color_attachment.set_load_action(MTLLoadAction::Clear);
                    color_attachment.set_clear_color(MTLClearColor::new(0.1, 0.1, 0.2, 1.0));
                    color_attachment.set_store_action(MTLStoreAction::Store);
                    
                    // Set up command buffer
                    let command_buffer = command_queue.new_command_buffer();
                    let command_encoder = command_buffer.new_render_command_encoder(&render_pass_descriptor);

                    // Record triangle draw call

                    println!("{}", vertex_buffer_data.len());
                    println!("{}", vertex_buffer.length());
                    println!("{}", vertex_buffer.allocated_size());
                    unsafe {
                        //&*(vertex_buffer.contents() as *const HelloTriangleVertex)
                        println!("{:?}", &*(vertex_buffer.contents() as *const HelloTriangleVertex));
                        println!("{:?}", *((vertex_buffer.contents() as *mut HelloTriangleVertex).wrapping_add(1)));
                        println!("{:?}", *((vertex_buffer.contents() as *mut HelloTriangleVertex).wrapping_add(2)));
                    }
                    command_encoder.set_render_pipeline_state(&hello_triangle_pipeline_state);
                    command_encoder.set_scissor_rect(MTLScissorRect{x: 0, y: 0, width: drawable_size.width as u64, height: drawable_size.height as u64});
                    command_encoder.set_vertex_buffer(0, Some(&vertex_buffer), 0);
                    command_encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, vertex_buffer_data.len() as u64);
                    command_encoder.end_encoding();

                    // Present framebuffer
                    command_buffer.present_drawable(&drawable);
                    command_buffer.commit();
                }
                _ => {}
            }
        });
    });
}
