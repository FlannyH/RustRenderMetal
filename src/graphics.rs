use std::f32::consts::PI;
use std::mem;
use std::path::Path;

use cocoa::appkit::NSView;
use cocoa::base::YES;
use core_graphics_types::geometry::CGSize;
use glam::Mat4;
use metal::{Device, MetalLayer, MTLPixelFormat, RenderPipelineState, RenderPipelineDescriptor, CommandQueue, Library, MTLResourceOptions, RenderPassDescriptor, MTLClearColor, MTLStoreAction, MTLScissorRect, MTLPrimitiveType, MTLViewport, Buffer};
use metal::foreign_types::ForeignType;
use winit::platform::macos::WindowExtMacOS;
use metal::MTLLoadAction;
use winit::window::Window;

use crate::mesh::{Mesh, Model};
use crate::structs::{Vertex, ConstBuffer, Transform};

// Todo: add transform
pub struct ModelQueueEntry {
    pub model_id: usize,
}

pub struct Renderer{
    pub device: Option<Device>,
    pipeline_state: Option<RenderPipelineState>,
    library: Option<Library>,
    command_queue: Option<CommandQueue>,
    layer: Option<MetalLayer>,
    const_buffer_gpu: Option<Buffer>,
    const_buffer_cpu: ConstBuffer,
    loaded_models: Vec<Model>,
    model_queue: Vec<ModelQueueEntry>,
}

impl Renderer{
    pub fn new(window: &Window) -> Self {
        // Initialize renderer with none
        let mut renderer = Renderer {
            device: None,
            pipeline_state: None,
            command_queue: None,
            library: None,
            layer: None,
            const_buffer_cpu: ConstBuffer{
                model_matrix: Mat4::IDENTITY,
                view_matrix: Mat4::IDENTITY,
                proj_matrix: Mat4::IDENTITY,
            },
            const_buffer_gpu: None,
            model_queue: Vec::new(),
            loaded_models: Vec::new(),
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

    pub fn begin_frame(&mut self) {
        self.model_queue.clear();
    }

    pub fn end_frame(&self) {
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
        command_encoder.set_vertex_buffer(1, Some(self.const_buffer_gpu.as_ref().unwrap()), 0);
        for model_id in &self.model_queue {
            let model = &self.loaded_models[model_id.model_id];
            for name in model.meshes.keys() {
                let mesh = model.meshes.get(name).unwrap();
                let _material = model.materials.get(name);
                command_encoder.set_vertex_buffer(0, Some(mesh.buffer.as_ref().unwrap()), 0);
                command_encoder.draw_primitives(MTLPrimitiveType::Triangle, 0, mesh.verts.len() as u64);
            }
        }
        command_encoder.end_encoding();

        // Present framebuffer
        command_buffer.present_drawable(drawable);
        command_buffer.commit();
    }

    pub fn resize_framebuffer(&mut self, width: u32, height: u32) {
        println!("framebuffer resized to {width}, {height}");
        self.layer.as_ref().unwrap().set_drawable_size(CGSize::new(width as f64, height as f64));
    }

    pub fn draw_model(&mut self, model_queue_entry: ModelQueueEntry) {
        self.model_queue.push(model_queue_entry);
    }

    pub fn load_model(&mut self, path: &Path) -> Option<usize> {
        let mut model = match Model::load_gltf(path) {
            Ok(mdl) => mdl,
            Err(s) => {println!("Error loading model \"{}\": {s}", path.display()); return None;}
        };
        
        for (name, mesh) in &mut model.meshes {
            println!("Uploading mesh {name}");
            self.upload_vertex_buffer(mesh);
        }

        self.loaded_models.push(model);
        return Some(self.loaded_models.len() - 1);
    }

    pub fn update_camera(&mut self, camera_transform: &Transform) {
        // Update CPU-side buffer
        self.const_buffer_cpu.view_matrix = camera_transform.view_matrix().transpose();
        self.const_buffer_cpu.proj_matrix = Mat4::perspective_rh_gl(PI / 4.0, 16.0 / 9.0, 0.1, 1000.0).transpose();

        // Update GPU-side buffer
        self.const_buffer_gpu = Some(self.device.as_ref().unwrap().new_buffer_with_data(
            &mut self.const_buffer_cpu as *mut _ as *const std::ffi::c_void,
            mem::size_of::<ConstBuffer>() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache | MTLResourceOptions::StorageModeManaged,
        ));
    }
}