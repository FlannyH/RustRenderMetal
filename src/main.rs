use std::fs::File;

use metal_rs::{MTLPixelFormat, CompileOptions};

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

    // Load shader source
    let mut shader_source_file = File::open("metal/shaders/hello_triangle.metal").expect("Failed to open shader file.");
    let mut source = String::new();
    file.read_to_string(&mut source).expect("Failed to read file.");

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


}
