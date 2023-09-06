use std::mem;

use cocoa::appkit::NSView;
use cocoa::base::YES;
use core_graphics_types::geometry::CGSize;
use metal::{Device, MetalLayer, MTLPixelFormat, RenderPipelineState, RenderPipelineDescriptor, CommandQueue, Library, MTLResourceOptions, RenderPassDescriptor, MTLClearColor, MTLStoreAction, MTLScissorRect, MTLPrimitiveType, MTLViewport, Buffer};
use metal::foreign_types::ForeignType;
use winit::platform::macos::WindowExtMacOS;
use metal::MTLLoadAction;
use winit::window::Window;

use crate::mesh::Mesh;
use crate::structs::{Vertex, ConstBuffer};

pub struct Renderer{
    pub device: Option<Device>,
    pipeline_state: Option<RenderPipelineState>,
    library: Option<Library>,
    command_queue: Option<CommandQueue>,
    layer: Option<MetalLayer>,
    const_buffer: Option<Buffer>,
}

impl Renderer {
    pub fn new(window: &Window) -> Self {
        // Initialize renderer with none
        let mut renderer = Renderer {
            device: None,
            pipeline_state: None,
            command_queue: None,
            library: None,
            layer: None,
            const_buffer: None,
        };

        // Create device
        renderer.device = Some(Device::system_default().expect("Could not create device."));

        // Create metal layer
        renderer.layer = Some(MetalLayer::new());
        renderer.layer.as_ref().unwrap().set_device(renderer.device.as_ref().unwrap());
        renderer.layer.as_ref().unwrap().set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        renderer.layer.as_ref().unwrap().set_presents_with_transaction(false);

        // Create view - a sort of canvas where you draw graphics using Metal commands
        unsafe {
            let view = window.ns_view() as cocoa::base::id;
            view.setWantsLayer(YES);
            view.setLayer(mem::transmute(renderer.layer.as_ref().unwrap().as_ptr()));
        }

        // Store the view size
        let drawable_size = window.inner_size();
        renderer.layer.as_ref().unwrap().set_drawable_size(CGSize::new(drawable_size.width as f64, drawable_size.height as f64));

        // Create command queue
        renderer.command_queue = Some(renderer.device.as_ref().unwrap().new_command_queue());

        return renderer;
    }

    pub fn load_library(&mut self, path: &str) {    
        self.library = Some(self.device.as_ref().unwrap().new_library_with_file(path).expect("Failed to load Metal library"));
    }

    pub fn prepare_pipeline_state (
        &mut self,
        vertex_shader_path: &str,
        fragment_shader_path: &str,
    ) {
        // Get compiled functions from the library
        let vertex_function = self.library.as_ref().unwrap().get_function(vertex_shader_path, None).unwrap();
        let fragment_function = self.library.as_ref().unwrap().get_function(fragment_shader_path, None).unwrap();

        // Create pipeline state descriptor - handles things like shader program, buffer to render to, blend mode, etc.
        let pipeline_state_desc = RenderPipelineDescriptor::new();
        pipeline_state_desc.set_vertex_function(Some(&vertex_function));
        pipeline_state_desc.set_fragment_function(Some(&fragment_function));

        let attachment = pipeline_state_desc.color_attachments().object_at(0).unwrap();
        attachment.set_pixel_format(MTLPixelFormat::RGBA8Unorm);
        attachment.set_blending_enabled(false);
        self.pipeline_state = Some(self.device.as_ref().unwrap().new_render_pipeline_state(&pipeline_state_desc).unwrap());
    }

    pub fn upload_vertex_buffer(&mut self, mesh: &mut Mesh) {
        // Create the vertex buffer on the device
        mesh.buffer = Some(self.device.as_ref().unwrap().new_buffer_with_data(
            mesh.verts.as_ptr() as *const _,
            (mesh.verts.len() * mem::size_of::<Vertex>()) as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        ));
    }

    pub fn upload_const_buffer(&mut self, const_buffer: &mut ConstBuffer) {
        // Create the constant buffer on the device
        self.const_buffer = Some(self.device.as_ref().unwrap().new_buffer_with_data(
            const_buffer as *mut _ as *const std::ffi::c_void,
            mem::size_of::<ConstBuffer>() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        ));
    }

    // mesh will be removed as parameter, it's here temporarily to have something working
    pub fn render_frame(&self, mesh: &Mesh) {
        // Get the next framebuffer
        let drawable = match self.layer.as_ref().unwrap().next_drawable() {
            Some(drawable) => drawable,
            None => return,
        };
        let size = self.layer.as_ref().unwrap().drawable_size();

        // Set up framebuffer
        let render_pass_descriptor = RenderPassDescriptor::new();
        let color_attachment = render_pass_descriptor.color_attachments().object_at(0).expect("Failed to get color attachment");
        color_attachment.set_texture(Some(drawable.texture()));
        color_attachment.set_load_action(MTLLoadAction::Clear);
        color_attachment.set_clear_color(MTLClearColor::new(0.1, 0.1, 0.2, 1.0));
        color_attachment.set_store_action(MTLStoreAction::Store);

        // Set up command buffer
        let command_buffer = self.command_queue.as_ref().unwrap().new_command_buffer();
        let command_encoder = command_buffer.new_render_command_encoder(render_pass_descriptor);

        // Record mesh draw call
        command_encoder.set_render_pipeline_state(self.pipeline_state.as_ref().unwrap().as_ref());
        command_encoder.set_scissor_rect(MTLScissorRect{x: 0, y: 0, width: size.width as u64, height: size.height as u64});
        command_encoder.set_viewport(MTLViewport{
            originX: 0.0,
            originY: 0.0,
            width: size.width as f64,
            height: size.height as f64,
            znear: -1.0,
            zfar: 1.0,
        });
        command_encoder.set_vertex_buffer(0, Some(mesh.buffer.as_ref().unwrap()), 0);
        command_encoder.set_vertex_buffer(1, Some(self.const_buffer.as_ref().unwrap()), 0);
        command_encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, mesh.verts.len() as u64);
        command_encoder.end_encoding();

        // Present framebuffer
        command_buffer.present_drawable(drawable);
        command_buffer.commit();
    }

    pub fn resize_framebuffer(&mut self, width: u32, height: u32) {
        println!("framebuffer resized to {width}, {height}");
        self.layer.as_ref().unwrap().set_drawable_size(CGSize::new(width as f64, height as f64));
    }

    pub fn draw_mesh(&self, _mesh: &Mesh) {
    }
}